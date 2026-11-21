use std::any::{Any, TypeId};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

/// A name of any type, compared and hashed with its type.
trait Name: Any {
    fn is(&self, other: &dyn Any) -> bool;
    fn hashed(&self, state: &mut dyn Hasher);
    fn as_any(&self) -> &dyn Any;
}

impl<K: Hash + Eq + 'static> Name for K {
    fn is(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<K>().is_some_and(|other| self == other)
    }

    fn hashed(&self, mut state: &mut dyn Hasher) {
        TypeId::of::<K>().hash(&mut state);
        self.hash(&mut state);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl PartialEq for dyn Name {
    fn eq(&self, other: &Self) -> bool {
        self.is(other.as_any())
    }
}

impl Eq for dyn Name {}

impl Hash for dyn Name {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashed(state);
    }
}

/// A name that is kept. It is compared and hashed as the name inside it, so that a name is found
/// by a borrow of it, with nothing allocated.
struct Kept(Box<dyn Name>);

impl PartialEq for Kept {
    fn eq(&self, other: &Self) -> bool {
        let (this, that): (&dyn Name, &dyn Name) = (&*self.0, &*other.0);
        this == that
    }
}

impl Eq for Kept {}

impl Hash for Kept {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self.0).hash(state);
    }
}

impl Borrow<dyn Name> for Kept {
    fn borrow(&self) -> &dyn Name {
        &*self.0
    }
}

struct Held<R> {
    reference: R,
    /// The drawing that last gave this name.
    drawn: u64,
}

/// The places a program names, as one platform's drawer keeps them for a window.
///
/// A program's logic names nothing that is drawn. Where it wants something of a place on the
/// screen - the keyboard to stand there - it asks as an effect, by a name of its own, and the
/// drawing gives that name to what it draws. A drawer keeps here, by name, its platform's
/// reference `R` to each thing a drawing names, the same reference for as long as each drawing
/// names it, and the newest ask until the drawing that follows it stands: that drawing may be the
/// one that first shows the place.
///
/// A name is any value that can be compared and hashed, such as a case of the program's own enum
/// of places. Names of different types are different names.
///
/// ```
/// use pamba::Places;
///
/// #[derive(PartialEq, Eq, Hash)]
/// enum Place {
///     Pay,
/// }
///
/// let mut places = Places::new();
/// // A step's effect asks for the keyboard at the press that pays.
/// places.ask(Place::Pay);
/// // The drawing that follows gives the name to a control, by the platform's reference to it.
/// places.begin();
/// let pay = places.named(Place::Pay, || "the control");
/// places.end();
/// // Once that drawing stands, the ask is carried out on the reference, once.
/// assert_eq!(places.take_asked(), Some(pay));
/// assert_eq!(places.take_asked(), None);
/// ```
pub struct Places<R> {
    held: HashMap<Kept, Held<R>>,
    /// The drawing being made, or the last one made.
    drawing: u64,
    asked: Option<Kept>,
    asks: u64,
}

impl<R> Places<R> {
    /// No place named, and nothing asked.
    #[must_use]
    pub fn new() -> Self {
        Self {
            held: HashMap::new(),
            drawing: 0,
            asked: None,
            asks: 0,
        }
    }

    /// A drawing starts: the names given from here to [`end`](Self::end) are this drawing's.
    pub fn begin(&mut self) {
        self.drawing += 1;
    }

    /// The reference for what this drawing calls `name`: the one the drawing before had for it,
    /// or `new`'s where that drawing did not give the name.
    ///
    /// A drawing gives one name to one thing: given to two, they share the reference.
    pub fn named<K>(&mut self, name: K, new: impl FnOnce() -> R) -> R
    where
        K: Hash + Eq + 'static,
        R: Clone,
    {
        let borrowed: &dyn Name = &name;
        if let Some(held) = self.held.get_mut(borrowed) {
            held.drawn = self.drawing;
            return held.reference.clone();
        }
        let reference = new();
        self.held.insert(
            Kept(Box::new(name)),
            Held {
                reference: reference.clone(),
                drawn: self.drawing,
            },
        );
        reference
    }

    /// The drawing is made: a name it did not give is dropped, with its reference.
    pub fn end(&mut self) {
        let drawing = self.drawing;
        self.held.retain(|_, held| held.drawn == drawing);
    }

    /// The program asks for the place called `name`. It replaces an ask not yet taken.
    pub fn ask<K: Hash + Eq + 'static>(&mut self, name: K) {
        self.asked = Some(Kept(Box::new(name)));
        self.asks += 1;
    }

    /// How many asks have been made: it changes exactly when there is a new ask to carry out, so
    /// a drawer that acts once a drawing stands compares it with the count it last acted on.
    #[must_use]
    pub fn asks(&self) -> u64 {
        self.asks
    }

    /// Whether an ask waits for the drawing that follows it.
    #[must_use]
    pub fn is_asked(&self) -> bool {
        self.asked.is_some()
    }

    /// Takes the newest ask, once the drawing that follows it stands, and gives the reference
    /// that drawing had for its name. Where the drawing gave no such name the ask comes to
    /// nothing: it is taken all the same, and a later drawing does not bring it back.
    pub fn take_asked(&mut self) -> Option<R>
    where
        R: Clone,
    {
        let asked = self.asked.take()?;
        self.held.get(&*asked.0).map(|held| held.reference.clone())
    }
}

impl<R> Default for Places<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> fmt::Debug for Places<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Places")
            .field("named", &self.held.len())
            .field("asked", &self.asked.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::Name;

    #[test]
    fn names_that_are_equal_values_of_different_types_are_different_names() {
        let (narrow, wide, again): (&dyn Name, &dyn Name, &dyn Name) = (&1_u32, &1_u64, &1_u32);

        assert!(
            narrow != wide,
            "a u32 and a u64 that are both 1 are one name"
        );
        assert!(
            narrow == again,
            "the same value of the same type is two names"
        );
    }
}
