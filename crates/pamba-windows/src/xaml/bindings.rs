#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutomationNotificationKind(pub i32);
impl AutomationNotificationKind {
    pub const Other: Self = Self(4);
}
impl windows_core::imp::TypeKind for AutomationNotificationKind {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AutomationNotificationKind {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.UI.Xaml.Automation.Peers.AutomationNotificationKind;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Xaml.Automation.Peers.AutomationNotificationKind",
    );
}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AutomationNotificationProcessing(pub i32);
impl AutomationNotificationProcessing {
    pub const All: Self = Self(2);
}
impl windows_core::imp::TypeKind for AutomationNotificationProcessing {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for AutomationNotificationProcessing {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"enum(Microsoft.UI.Xaml.Automation.Peers.AutomationNotificationProcessing;i4)",
    );
    const NAME: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::from_slice(
        b"Microsoft.UI.Xaml.Automation.Peers.AutomationNotificationProcessing",
    );
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationPeer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AutomationPeer,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(AutomationPeer, DependencyObject);
impl AutomationPeer {
    pub fn RaiseNotificationEvent(
        &self,
        notificationkind: AutomationNotificationKind,
        notificationprocessing: AutomationNotificationProcessing,
        displaystring: &windows_core::HSTRING,
        activityid: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).RaiseNotificationEvent)(
                windows_core::Interface::as_raw(self),
                notificationkind,
                notificationprocessing,
                core::mem::transmute_copy(displaystring),
                core::mem::transmute_copy(activityid),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for AutomationPeer {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAutomationPeer>();
}
unsafe impl windows_core::Interface for AutomationPeer {
    type Vtable = <IAutomationPeer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAutomationPeer as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for AutomationPeer {
    const NAME: &'static str = "Microsoft.UI.Xaml.Automation.Peers.AutomationPeer";
}
unsafe impl Send for AutomationPeer {}
unsafe impl Sync for AutomationPeer {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AutomationProperties(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    AutomationProperties,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl AutomationProperties {
    pub fn GetAutomationId<P0>(element: P0) -> windows_core::Result<windows_core::HSTRING>
    where
        P0: windows_core::Param<DependencyObject>,
    {
        Self::IAutomationPropertiesStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetAutomationId)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .map(|| core::mem::transmute(result__))
        })
    }
    fn IAutomationPropertiesStatics<
        R,
        F: FnOnce(&IAutomationPropertiesStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            AutomationProperties,
            IAutomationPropertiesStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for AutomationProperties {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IAutomationProperties>();
}
unsafe impl windows_core::Interface for AutomationProperties {
    type Vtable = <IAutomationProperties as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IAutomationProperties as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for AutomationProperties {
    const NAME: &'static str = "Microsoft.UI.Xaml.Automation.AutomationProperties";
}
unsafe impl Send for AutomationProperties {}
unsafe impl Sync for AutomationProperties {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BringIntoViewOptions(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    BringIntoViewOptions,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl BringIntoViewOptions {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            BringIntoViewOptions,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn SetTargetRect(&self, value: Option<Rect>) -> windows_core::Result<()> {
        let value__ = value.map(<windows_reference::IReference<Rect> as From<_>>::from);
        unsafe {
            (windows_core::Interface::vtable(self).SetTargetRect)(
                windows_core::Interface::as_raw(self),
                windows_core::Param::param(value__.as_ref()).abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for BringIntoViewOptions {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IBringIntoViewOptions>();
}
unsafe impl windows_core::Interface for BringIntoViewOptions {
    type Vtable = <IBringIntoViewOptions as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IBringIntoViewOptions as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for BringIntoViewOptions {
    const NAME: &'static str = "Microsoft.UI.Xaml.BringIntoViewOptions";
}
unsafe impl Send for BringIntoViewOptions {}
unsafe impl Sync for BringIntoViewOptions {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentControl(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ContentControl,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ContentControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl ContentControl {
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualWidth)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualHeight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).XamlRoot)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).UpdateLayout)(windows_core::Interface::as_raw(
                this,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(this),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ContentControl {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IContentControl>();
}
unsafe impl windows_core::Interface for ContentControl {
    type Vtable = <IContentControl as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IContentControl as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for ContentControl {
    const NAME: &'static str = "Microsoft.UI.Xaml.Controls.ContentControl";
}
unsafe impl Send for ContentControl {}
unsafe impl Sync for ContentControl {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Control(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    Control,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(Control, FrameworkElement, UIElement, DependencyObject);
impl Control {
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualWidth)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualHeight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).XamlRoot)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).UpdateLayout)(windows_core::Interface::as_raw(
                this,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(this),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for Control {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IControl>();
}
unsafe impl windows_core::Interface for Control {
    type Vtable = <IControl as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IControl as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for Control {
    const NAME: &'static str = "Microsoft.UI.Xaml.Controls.Control";
}
unsafe impl Send for Control {}
unsafe impl Sync for Control {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyObject(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    DependencyObject,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for DependencyObject {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IDependencyObject>();
}
unsafe impl windows_core::Interface for DependencyObject {
    type Vtable = <IDependencyObject as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IDependencyObject as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for DependencyObject {
    const NAME: &'static str = "Microsoft.UI.Xaml.DependencyObject";
}
unsafe impl Send for DependencyObject {}
unsafe impl Sync for DependencyObject {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventHandler<T>(windows_core::IUnknown, core::marker::PhantomData<T>)
where
    T: windows_core::RuntimeType + 'static;
unsafe impl<T: windows_core::RuntimeType + 'static> windows_core::Interface for EventHandler<T> {
    type Vtable = EventHandler_Vtbl<T>;
    const IID: windows_core::GUID =
        windows_core::GUID::from_signature(<Self as windows_core::RuntimeType>::SIGNATURE);
}
impl<T: windows_core::RuntimeType + 'static> windows_core::RuntimeType for EventHandler<T> {
    const SIGNATURE: windows_core::imp::ConstBuffer = windows_core::imp::ConstBuffer::new()
        .push_slice(b"pinterface({9de1c535-6ae1-11e0-84e1-18a905bcc53f}")
        .push_slice(b";")
        .push_other(T::SIGNATURE)
        .push_slice(b")");
}
impl<T: windows_core::RuntimeType + 'static> EventHandler<T> {
    pub fn new<
        F: Fn(
                windows_core::Ref<windows_core::IInspectable>,
                windows_core::Ref<T>,
            ) -> windows_core::Result<()>
            + Send
            + Sync
            + 'static,
    >(
        invoke: F,
    ) -> Self {
        let com = windows_core::imp::DelegateBox::<Self, F>::new(
            &EventHandlerBox::<T, F>::VTABLE,
            invoke,
        );
        unsafe { core::mem::transmute(windows_core::imp::box_new(com)) }
    }
    pub fn Invoke<P0, P1>(&self, sender: P0, args: P1) -> windows_core::Result<()>
    where
        P0: windows_core::Param<windows_core::IInspectable>,
        P1: windows_core::Param<T>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).Invoke)(
                windows_core::Interface::as_raw(self),
                sender.param().abi(),
                args.param().abi(),
            )
            .ok()
        }
    }
}
#[repr(C)]
pub struct EventHandler_Vtbl<T>
where
    T: windows_core::RuntimeType + 'static,
{
    base__: windows_core::IUnknown_Vtbl,
    Invoke: unsafe extern "system" fn(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        args: windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT,
    T: core::marker::PhantomData<T>,
}
struct EventHandlerBox<
    T,
    F: Fn(
            windows_core::Ref<windows_core::IInspectable>,
            windows_core::Ref<T>,
        ) -> windows_core::Result<()>
        + Send
        + Sync
        + 'static,
>(core::marker::PhantomData<(T, fn() -> F)>)
where
    T: windows_core::RuntimeType + 'static;
impl<
    T: windows_core::RuntimeType + 'static,
    F: Fn(
            windows_core::Ref<windows_core::IInspectable>,
            windows_core::Ref<T>,
        ) -> windows_core::Result<()>
        + Send
        + Sync
        + 'static,
> EventHandlerBox<T, F>
{
    const VTABLE: EventHandler_Vtbl<T> = EventHandler_Vtbl::<T> {
        base__: windows_core::IUnknown_Vtbl {
            QueryInterface: windows_core::imp::DelegateBox::<EventHandler<T>, F>::QueryInterface,
            AddRef: windows_core::imp::DelegateBox::<EventHandler<T>, F>::AddRef,
            Release: windows_core::imp::DelegateBox::<EventHandler<T>, F>::Release,
        },
        Invoke: Self::Invoke,
        T: core::marker::PhantomData::<T>,
    };
    unsafe extern "system" fn Invoke(
        this: *mut core::ffi::c_void,
        sender: *mut core::ffi::c_void,
        args: windows_core::imp::AbiType<T>,
    ) -> windows_core::HRESULT {
        unsafe {
            let this = &*(this as *mut *mut core::ffi::c_void
                as *mut windows_core::imp::DelegateBox<EventHandler<T>, F>);
            (this.invoke)(
                core::mem::transmute_copy(&sender),
                core::mem::transmute_copy(&args),
            )
            .into()
        }
    }
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusManager(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FocusManager,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl FocusManager {
    pub fn GotFocus<F>(handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<windows_core::IInspectable>,
                windows_core::Ref<FocusManagerGotFocusEventArgs>,
            ) + Send
            + Sync
            + 'static,
    {
        let handler = <EventHandler<FocusManagerGotFocusEventArgs>>::new(move |a0, a1| {
            handler(a0, a1);
            Ok(())
        });
        Self::IFocusManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(this).GotFocus)(
                windows_core::Interface::as_raw(this),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                this.clone(),
                token__,
                windows_core::Interface::vtable(this).RemoveGotFocus,
            ))
        })
    }
    pub fn LosingFocus<F>(handler: F) -> windows_core::Result<windows_core::EventRevoker>
    where
        F: Fn(
                windows_core::Ref<windows_core::IInspectable>,
                windows_core::Ref<LosingFocusEventArgs>,
            ) + Send
            + Sync
            + 'static,
    {
        let handler = <EventHandler<LosingFocusEventArgs>>::new(move |a0, a1| {
            handler(a0, a1);
            Ok(())
        });
        Self::IFocusManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            let token__ = (windows_core::Interface::vtable(this).LosingFocus)(
                windows_core::Interface::as_raw(this),
                windows_core::Interface::as_raw(&handler),
                &mut result__,
            )
            .map(|| result__)?;
            Ok(windows_core::EventRevoker::new(
                this.clone(),
                token__,
                windows_core::Interface::vtable(this).RemoveLosingFocus,
            ))
        })
    }
    pub fn GetFocusedElementWithRoot<P0>(
        xamlroot: P0,
    ) -> windows_core::Result<windows_core::IInspectable>
    where
        P0: windows_core::Param<XamlRoot>,
    {
        Self::IFocusManagerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetFocusedElementWithRoot)(
                windows_core::Interface::as_raw(this),
                xamlroot.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        })
    }
    fn IFocusManagerStatics<R, F: FnOnce(&IFocusManagerStatics) -> windows_core::Result<R>>(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<FocusManager, IFocusManagerStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for FocusManager {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFocusManager>();
}
unsafe impl windows_core::Interface for FocusManager {
    type Vtable = <IFocusManager as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFocusManager as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for FocusManager {
    const NAME: &'static str = "Microsoft.UI.Xaml.Input.FocusManager";
}
unsafe impl Send for FocusManager {}
unsafe impl Sync for FocusManager {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FocusManagerGotFocusEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FocusManagerGotFocusEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl FocusManagerGotFocusEventArgs {
    pub fn NewFocusedElement(&self) -> windows_core::Result<DependencyObject> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NewFocusedElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for FocusManagerGotFocusEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFocusManagerGotFocusEventArgs>();
}
unsafe impl windows_core::Interface for FocusManagerGotFocusEventArgs {
    type Vtable = <IFocusManagerGotFocusEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IFocusManagerGotFocusEventArgs as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for FocusManagerGotFocusEventArgs {
    const NAME: &'static str = "Microsoft.UI.Xaml.Input.FocusManagerGotFocusEventArgs";
}
unsafe impl Send for FocusManagerGotFocusEventArgs {}
unsafe impl Sync for FocusManagerGotFocusEventArgs {}
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FocusState(pub i32);
impl FocusState {
    pub const Unfocused: Self = Self(0);
    pub const Pointer: Self = Self(1);
    pub const Keyboard: Self = Self(2);
    pub const Programmatic: Self = Self(3);
}
impl windows_core::imp::TypeKind for FocusState {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for FocusState {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"enum(Microsoft.UI.Xaml.FocusState;i4)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Microsoft.UI.Xaml.FocusState");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameworkElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FrameworkElement,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(FrameworkElement, UIElement, DependencyObject);
impl FrameworkElement {
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualWidth)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ActualHeight)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).XamlRoot)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).UpdateLayout)(windows_core::Interface::as_raw(
                this,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(this),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for FrameworkElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFrameworkElement>();
}
unsafe impl windows_core::Interface for FrameworkElement {
    type Vtable = <IFrameworkElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IFrameworkElement as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for FrameworkElement {
    const NAME: &'static str = "Microsoft.UI.Xaml.FrameworkElement";
}
unsafe impl Send for FrameworkElement {}
unsafe impl Sync for FrameworkElement {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrameworkElementAutomationPeer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    FrameworkElementAutomationPeer,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    FrameworkElementAutomationPeer,
    AutomationPeer,
    DependencyObject
);
impl FrameworkElementAutomationPeer {
    pub fn RaiseNotificationEvent(
        &self,
        notificationkind: AutomationNotificationKind,
        notificationprocessing: AutomationNotificationProcessing,
        displaystring: &windows_core::HSTRING,
        activityid: &windows_core::HSTRING,
    ) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IAutomationPeer>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).RaiseNotificationEvent)(
                windows_core::Interface::as_raw(this),
                notificationkind,
                notificationprocessing,
                core::mem::transmute_copy(displaystring),
                core::mem::transmute_copy(activityid),
            )
            .ok()
        }
    }
    pub fn CreatePeerForElement<P0>(element: P0) -> windows_core::Result<AutomationPeer>
    where
        P0: windows_core::Param<UIElement>,
    {
        Self::IFrameworkElementAutomationPeerStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).CreatePeerForElement)(
                windows_core::Interface::as_raw(this),
                element.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        })
    }
    fn IFrameworkElementAutomationPeerStatics<
        R,
        F: FnOnce(&IFrameworkElementAutomationPeerStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            FrameworkElementAutomationPeer,
            IFrameworkElementAutomationPeerStatics,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for FrameworkElementAutomationPeer {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IFrameworkElementAutomationPeer>();
}
unsafe impl windows_core::Interface for FrameworkElementAutomationPeer {
    type Vtable = <IFrameworkElementAutomationPeer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID =
        <IFrameworkElementAutomationPeer as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for FrameworkElementAutomationPeer {
    const NAME: &'static str = "Microsoft.UI.Xaml.Automation.Peers.FrameworkElementAutomationPeer";
}
unsafe impl Send for FrameworkElementAutomationPeer {}
unsafe impl Sync for FrameworkElementAutomationPeer {}
windows_core::imp::define_winrt_interface!(
    IAutomationPeer,
    IAutomationPeer_Vtbl,
    0xe51d3e4e_34f0_568c_999f_6277e2afe6d7,
    b"Microsoft.UI.Xaml.Automation.Peers.IAutomationPeer"
);
#[repr(C)]
pub struct IAutomationPeer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    EventsSource: usize,
    SetEventsSource: usize,
    GetPattern: usize,
    RaiseAutomationEvent: usize,
    RaisePropertyChangedEvent: usize,
    GetAcceleratorKey: usize,
    GetAccessKey: usize,
    GetAutomationControlType: usize,
    GetAutomationId: usize,
    GetBoundingRectangle: usize,
    GetChildren: usize,
    Navigate: usize,
    GetClassName: usize,
    GetClickablePoint: usize,
    GetHelpText: usize,
    GetItemStatus: usize,
    GetItemType: usize,
    GetLabeledBy: usize,
    GetLocalizedControlType: usize,
    GetName: usize,
    GetOrientation: usize,
    HasKeyboardFocus: usize,
    IsContentElement: usize,
    IsControlElement: usize,
    IsEnabled: usize,
    IsKeyboardFocusable: usize,
    IsOffscreen: usize,
    IsPassword: usize,
    IsRequiredForForm: usize,
    SetFocus: usize,
    GetParent: usize,
    InvalidatePeer: usize,
    GetPeerFromPoint: usize,
    GetElementFromPoint: usize,
    GetFocusedElement: usize,
    GetLiveSetting: usize,
    ShowContextMenu: usize,
    GetControlledPeers: usize,
    GetAnnotations: usize,
    SetParent: usize,
    RaiseTextEditTextChangedEvent: usize,
    GetPositionInSet: usize,
    GetSizeOfSet: usize,
    GetLevel: usize,
    RaiseStructureChangedEvent: usize,
    GetLandmarkType: usize,
    GetLocalizedLandmarkType: usize,
    IsPeripheral: usize,
    IsDataValidForForm: usize,
    GetFullDescription: usize,
    GetCulture: usize,
    pub RaiseNotificationEvent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        AutomationNotificationKind,
        AutomationNotificationProcessing,
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IAutomationProperties,
    IAutomationProperties_Vtbl,
    0x525c6a71_dd8a_52a0_977b_db1b02f8e896,
    b"Microsoft.UI.Xaml.Automation.IAutomationProperties"
);
#[repr(C)]
pub struct IAutomationProperties_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IAutomationPropertiesStatics,
    IAutomationPropertiesStatics_Vtbl,
    0xb1e3e0f3_112f_5966_87dc_7862d4ad50e5,
    b"Microsoft.UI.Xaml.Automation.IAutomationPropertiesStatics"
);
#[repr(C)]
pub struct IAutomationPropertiesStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    AcceleratorKeyProperty: usize,
    GetAcceleratorKey: usize,
    SetAcceleratorKey: usize,
    AccessKeyProperty: usize,
    GetAccessKey: usize,
    SetAccessKey: usize,
    AutomationIdProperty: usize,
    pub GetAutomationId: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IBringIntoViewOptions,
    IBringIntoViewOptions_Vtbl,
    0xeeb4a447_eb9e_5003_a479_b9e3a886b708,
    b"Microsoft.UI.Xaml.IBringIntoViewOptions"
);
#[repr(C)]
pub struct IBringIntoViewOptions_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    AnimationDesired: usize,
    SetAnimationDesired: usize,
    TargetRect: usize,
    pub SetTargetRect: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IContentControl,
    IContentControl_Vtbl,
    0x07e81761_11b2_52ae_8f8b_4d53d2b5900a,
    b"Microsoft.UI.Xaml.Controls.IContentControl"
);
#[repr(C)]
pub struct IContentControl_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IControl,
    IControl_Vtbl,
    0x857d6e8a_d45a_5c69_a99c_bf6a5c54fb38,
    b"Microsoft.UI.Xaml.Controls.IControl"
);
#[repr(C)]
pub struct IControl_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IDependencyObject,
    IDependencyObject_Vtbl,
    0xe7beaee7_160e_50f7_8789_d63463f979fa,
    b"Microsoft.UI.Xaml.IDependencyObject"
);
#[repr(C)]
pub struct IDependencyObject_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IFocusManager,
    IFocusManager_Vtbl,
    0x9fd07bc5_d2d4_53fe_a31a_846de8b7a257,
    b"Microsoft.UI.Xaml.Input.IFocusManager"
);
#[repr(C)]
pub struct IFocusManager_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IFocusManagerGotFocusEventArgs,
    IFocusManagerGotFocusEventArgs_Vtbl,
    0x50aca341_4519_59cf_83b1_c9c45cfdb816,
    b"Microsoft.UI.Xaml.Input.IFocusManagerGotFocusEventArgs"
);
#[repr(C)]
pub struct IFocusManagerGotFocusEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub NewFocusedElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IFocusManagerStatics,
    IFocusManagerStatics_Vtbl,
    0xe73dce04_e23a_5fb3_96ab_7df04c51dff2,
    b"Microsoft.UI.Xaml.Input.IFocusManagerStatics"
);
#[repr(C)]
pub struct IFocusManagerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub GotFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i64,
    ) -> windows_core::HRESULT,
    pub RemoveGotFocus:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    LostFocus: usize,
    RemoveLostFocus: usize,
    GettingFocus: usize,
    RemoveGettingFocus: usize,
    pub LosingFocus: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i64,
    ) -> windows_core::HRESULT,
    pub RemoveLosingFocus:
        unsafe extern "system" fn(*mut core::ffi::c_void, i64) -> windows_core::HRESULT,
    TryFocusAsync: usize,
    TryMoveFocusAsync: usize,
    TryMoveFocusWithOptionsAsync: usize,
    TryMoveFocusWithOptions: usize,
    FindNextElement: usize,
    FindFirstFocusableElement: usize,
    FindLastFocusableElement: usize,
    FindNextElementWithOptions: usize,
    FindNextFocusableElement: usize,
    FindNextFocusableElementWithHint: usize,
    TryMoveFocus: usize,
    GetFocusedElement: usize,
    pub GetFocusedElementWithRoot: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IFrameworkElement,
    IFrameworkElement_Vtbl,
    0xfe08f13d_dc6a_5495_ad44_c2d8d21863b0,
    b"Microsoft.UI.Xaml.IFrameworkElement"
);
#[repr(C)]
pub struct IFrameworkElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Triggers: usize,
    Resources: usize,
    SetResources: usize,
    Tag: usize,
    SetTag: usize,
    Language: usize,
    SetLanguage: usize,
    pub ActualWidth:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub ActualHeight:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IFrameworkElementAutomationPeer,
    IFrameworkElementAutomationPeer_Vtbl,
    0x7dab4f24_605c_51cb_87db_3eed1b9fb37b,
    b"Microsoft.UI.Xaml.Automation.Peers.IFrameworkElementAutomationPeer"
);
#[repr(C)]
pub struct IFrameworkElementAutomationPeer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IFrameworkElementAutomationPeerStatics,
    IFrameworkElementAutomationPeerStatics_Vtbl,
    0x081f6fbe_6500_528a_a506_f5a4d41ddf6c,
    b"Microsoft.UI.Xaml.Automation.Peers.IFrameworkElementAutomationPeerStatics"
);
#[repr(C)]
pub struct IFrameworkElementAutomationPeerStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    FromElement: usize,
    pub CreatePeerForElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    ILosingFocusEventArgs,
    ILosingFocusEventArgs_Vtbl,
    0xfa0e5ffa_2b1b_52f8_bb66_e35f51e73cf3,
    b"Microsoft.UI.Xaml.Input.ILosingFocusEventArgs"
);
#[repr(C)]
pub struct ILosingFocusEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub OldFocusedElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub NewFocusedElement: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    SetNewFocusedElement: usize,
    pub FocusState:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut FocusState) -> windows_core::HRESULT,
    Direction: usize,
    Handled: usize,
    SetHandled: usize,
    InputDevice: usize,
    Cancel: usize,
    SetCancel: usize,
    CorrelationId: usize,
    pub TryCancel:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut bool) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IRoutedEventArgs,
    IRoutedEventArgs_Vtbl,
    0x0908c407_1c7d_5de3_9c50_d971c62ec8ec,
    b"Microsoft.UI.Xaml.IRoutedEventArgs"
);
#[repr(C)]
pub struct IRoutedEventArgs_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IScrollView,
    IScrollView_Vtbl,
    0x8c98c86d_378a_5102_a1e3_3352280fa010,
    b"Microsoft.UI.Xaml.Controls.IScrollView"
);
#[repr(C)]
pub struct IScrollView_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    Content: usize,
    SetContent: usize,
    CurrentAnchor: usize,
    ScrollPresenter: usize,
    ExpressionAnimationSources: usize,
    HorizontalOffset: usize,
    VerticalOffset: usize,
    ZoomFactor: usize,
    ExtentWidth: usize,
    ExtentHeight: usize,
    pub ViewportWidth:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    pub ViewportHeight:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IScrollViewer,
    IScrollViewer_Vtbl,
    0x1dc28c2e_996c_5394_89c3_4dc656b4ad46,
    b"Microsoft.UI.Xaml.Controls.IScrollViewer"
);
#[repr(C)]
pub struct IScrollViewer_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    HorizontalScrollBarVisibility: usize,
    SetHorizontalScrollBarVisibility: usize,
    VerticalScrollBarVisibility: usize,
    SetVerticalScrollBarVisibility: usize,
    IsHorizontalRailEnabled: usize,
    SetIsHorizontalRailEnabled: usize,
    IsVerticalRailEnabled: usize,
    SetIsVerticalRailEnabled: usize,
    IsHorizontalScrollChainingEnabled: usize,
    SetIsHorizontalScrollChainingEnabled: usize,
    IsVerticalScrollChainingEnabled: usize,
    SetIsVerticalScrollChainingEnabled: usize,
    IsZoomChainingEnabled: usize,
    SetIsZoomChainingEnabled: usize,
    IsScrollInertiaEnabled: usize,
    SetIsScrollInertiaEnabled: usize,
    IsZoomInertiaEnabled: usize,
    SetIsZoomInertiaEnabled: usize,
    HorizontalScrollMode: usize,
    SetHorizontalScrollMode: usize,
    VerticalScrollMode: usize,
    SetVerticalScrollMode: usize,
    ZoomMode: usize,
    SetZoomMode: usize,
    HorizontalSnapPointsAlignment: usize,
    SetHorizontalSnapPointsAlignment: usize,
    VerticalSnapPointsAlignment: usize,
    SetVerticalSnapPointsAlignment: usize,
    HorizontalSnapPointsType: usize,
    SetHorizontalSnapPointsType: usize,
    VerticalSnapPointsType: usize,
    SetVerticalSnapPointsType: usize,
    ZoomSnapPointsType: usize,
    SetZoomSnapPointsType: usize,
    HorizontalOffset: usize,
    pub ViewportWidth:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
    ScrollableWidth: usize,
    ComputedHorizontalScrollBarVisibility: usize,
    ExtentWidth: usize,
    VerticalOffset: usize,
    pub ViewportHeight:
        unsafe extern "system" fn(*mut core::ffi::c_void, *mut f64) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IUIElement,
    IUIElement_Vtbl,
    0xc3c01020_320c_5cf6_9d24_d396bbfa4d8b,
    b"Microsoft.UI.Xaml.IUIElement"
);
#[repr(C)]
pub struct IUIElement_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    DesiredSize: usize,
    AllowDrop: usize,
    SetAllowDrop: usize,
    Opacity: usize,
    SetOpacity: usize,
    Clip: usize,
    SetClip: usize,
    RenderTransform: usize,
    SetRenderTransform: usize,
    Projection: usize,
    SetProjection: usize,
    Transform3D: usize,
    SetTransform3D: usize,
    RenderTransformOrigin: usize,
    SetRenderTransformOrigin: usize,
    IsHitTestVisible: usize,
    SetIsHitTestVisible: usize,
    Visibility: usize,
    SetVisibility: usize,
    RenderSize: usize,
    UseLayoutRounding: usize,
    SetUseLayoutRounding: usize,
    Transitions: usize,
    SetTransitions: usize,
    CacheMode: usize,
    SetCacheMode: usize,
    IsTapEnabled: usize,
    SetIsTapEnabled: usize,
    IsDoubleTapEnabled: usize,
    SetIsDoubleTapEnabled: usize,
    CanDrag: usize,
    SetCanDrag: usize,
    IsRightTapEnabled: usize,
    SetIsRightTapEnabled: usize,
    IsHoldingEnabled: usize,
    SetIsHoldingEnabled: usize,
    ManipulationMode: usize,
    SetManipulationMode: usize,
    PointerCaptures: usize,
    ContextFlyout: usize,
    SetContextFlyout: usize,
    CompositeMode: usize,
    SetCompositeMode: usize,
    Lights: usize,
    CanBeScrollAnchor: usize,
    SetCanBeScrollAnchor: usize,
    ExitDisplayModeOnAccessKeyInvoked: usize,
    SetExitDisplayModeOnAccessKeyInvoked: usize,
    IsAccessKeyScope: usize,
    SetIsAccessKeyScope: usize,
    AccessKeyScopeOwner: usize,
    SetAccessKeyScopeOwner: usize,
    AccessKey: usize,
    SetAccessKey: usize,
    KeyTipPlacementMode: usize,
    SetKeyTipPlacementMode: usize,
    KeyTipHorizontalOffset: usize,
    SetKeyTipHorizontalOffset: usize,
    KeyTipVerticalOffset: usize,
    SetKeyTipVerticalOffset: usize,
    KeyTipTarget: usize,
    SetKeyTipTarget: usize,
    XYFocusKeyboardNavigation: usize,
    SetXYFocusKeyboardNavigation: usize,
    XYFocusUpNavigationStrategy: usize,
    SetXYFocusUpNavigationStrategy: usize,
    XYFocusDownNavigationStrategy: usize,
    SetXYFocusDownNavigationStrategy: usize,
    XYFocusLeftNavigationStrategy: usize,
    SetXYFocusLeftNavigationStrategy: usize,
    XYFocusRightNavigationStrategy: usize,
    SetXYFocusRightNavigationStrategy: usize,
    KeyboardAccelerators: usize,
    KeyboardAcceleratorPlacementTarget: usize,
    SetKeyboardAcceleratorPlacementTarget: usize,
    KeyboardAcceleratorPlacementMode: usize,
    SetKeyboardAcceleratorPlacementMode: usize,
    HighContrastAdjustment: usize,
    SetHighContrastAdjustment: usize,
    TabFocusNavigation: usize,
    SetTabFocusNavigation: usize,
    OpacityTransition: usize,
    SetOpacityTransition: usize,
    Translation: usize,
    SetTranslation: usize,
    TranslationTransition: usize,
    SetTranslationTransition: usize,
    Rotation: usize,
    SetRotation: usize,
    RotationTransition: usize,
    SetRotationTransition: usize,
    Scale: usize,
    SetScale: usize,
    ScaleTransition: usize,
    SetScaleTransition: usize,
    TransformMatrix: usize,
    SetTransformMatrix: usize,
    CenterPoint: usize,
    SetCenterPoint: usize,
    RotationAxis: usize,
    SetRotationAxis: usize,
    ActualOffset: usize,
    ActualSize: usize,
    pub XamlRoot: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    SetXamlRoot: usize,
    Shadow: usize,
    SetShadow: usize,
    RasterizationScale: usize,
    SetRasterizationScale: usize,
    FocusState: usize,
    UseSystemFocusVisuals: usize,
    SetUseSystemFocusVisuals: usize,
    XYFocusLeft: usize,
    SetXYFocusLeft: usize,
    XYFocusRight: usize,
    SetXYFocusRight: usize,
    XYFocusUp: usize,
    SetXYFocusUp: usize,
    XYFocusDown: usize,
    SetXYFocusDown: usize,
    IsTabStop: usize,
    SetIsTabStop: usize,
    TabIndex: usize,
    SetTabIndex: usize,
    KeyUp: usize,
    RemoveKeyUp: usize,
    KeyDown: usize,
    RemoveKeyDown: usize,
    GotFocus: usize,
    RemoveGotFocus: usize,
    LostFocus: usize,
    RemoveLostFocus: usize,
    DragStarting: usize,
    RemoveDragStarting: usize,
    DropCompleted: usize,
    RemoveDropCompleted: usize,
    CharacterReceived: usize,
    RemoveCharacterReceived: usize,
    DragEnter: usize,
    RemoveDragEnter: usize,
    DragLeave: usize,
    RemoveDragLeave: usize,
    DragOver: usize,
    RemoveDragOver: usize,
    Drop: usize,
    RemoveDrop: usize,
    PointerPressed: usize,
    RemovePointerPressed: usize,
    PointerMoved: usize,
    RemovePointerMoved: usize,
    PointerReleased: usize,
    RemovePointerReleased: usize,
    PointerEntered: usize,
    RemovePointerEntered: usize,
    PointerExited: usize,
    RemovePointerExited: usize,
    PointerCaptureLost: usize,
    RemovePointerCaptureLost: usize,
    PointerCanceled: usize,
    RemovePointerCanceled: usize,
    PointerWheelChanged: usize,
    RemovePointerWheelChanged: usize,
    Tapped: usize,
    RemoveTapped: usize,
    DoubleTapped: usize,
    RemoveDoubleTapped: usize,
    Holding: usize,
    RemoveHolding: usize,
    ContextRequested: usize,
    RemoveContextRequested: usize,
    ContextCanceled: usize,
    RemoveContextCanceled: usize,
    RightTapped: usize,
    RemoveRightTapped: usize,
    ManipulationStarting: usize,
    RemoveManipulationStarting: usize,
    ManipulationInertiaStarting: usize,
    RemoveManipulationInertiaStarting: usize,
    ManipulationStarted: usize,
    RemoveManipulationStarted: usize,
    ManipulationDelta: usize,
    RemoveManipulationDelta: usize,
    ManipulationCompleted: usize,
    RemoveManipulationCompleted: usize,
    AccessKeyDisplayRequested: usize,
    RemoveAccessKeyDisplayRequested: usize,
    AccessKeyDisplayDismissed: usize,
    RemoveAccessKeyDisplayDismissed: usize,
    AccessKeyInvoked: usize,
    RemoveAccessKeyInvoked: usize,
    ProcessKeyboardAccelerators: usize,
    RemoveProcessKeyboardAccelerators: usize,
    GettingFocus: usize,
    RemoveGettingFocus: usize,
    LosingFocus: usize,
    RemoveLosingFocus: usize,
    NoFocusCandidateFound: usize,
    RemoveNoFocusCandidateFound: usize,
    PreviewKeyDown: usize,
    RemovePreviewKeyDown: usize,
    PreviewKeyUp: usize,
    RemovePreviewKeyUp: usize,
    BringIntoViewRequested: usize,
    RemoveBringIntoViewRequested: usize,
    Measure: usize,
    Arrange: usize,
    CapturePointer: usize,
    ReleasePointerCapture: usize,
    ReleasePointerCaptures: usize,
    AddHandler: usize,
    RemoveHandler: usize,
    TransformToVisual: usize,
    InvalidateMeasure: usize,
    InvalidateArrange: usize,
    pub UpdateLayout: unsafe extern "system" fn(*mut core::ffi::c_void) -> windows_core::HRESULT,
    CancelDirectManipulations: usize,
    StartDragAsync: usize,
    StartBringIntoView: usize,
    pub StartBringIntoViewWithOptions: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IVisualTreeHelper,
    IVisualTreeHelper_Vtbl,
    0x5f69ac1e_6504_5e3f_a11c_87684c1db814,
    b"Microsoft.UI.Xaml.Media.IVisualTreeHelper"
);
#[repr(C)]
pub struct IVisualTreeHelper_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
}
windows_core::imp::define_winrt_interface!(
    IVisualTreeHelperStatics,
    IVisualTreeHelperStatics_Vtbl,
    0x5aece43c_7651_5bb5_855c_2198496e455e,
    b"Microsoft.UI.Xaml.Media.IVisualTreeHelperStatics"
);
#[repr(C)]
pub struct IVisualTreeHelperStatics_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    FindElementsInHostCoordinatesPoint: usize,
    FindElementsInHostCoordinatesRect: usize,
    FindAllElementsInHostCoordinatesPoint: usize,
    FindAllElementsInHostCoordinatesRect: usize,
    pub GetChild: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        i32,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
    pub GetChildrenCount: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut i32,
    ) -> windows_core::HRESULT,
    pub GetParent: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
windows_core::imp::define_winrt_interface!(
    IXamlRoot,
    IXamlRoot_Vtbl,
    0x60cb215a_ad15_520a_8b01_4416824f0441,
    b"Microsoft.UI.Xaml.IXamlRoot"
);
#[repr(C)]
pub struct IXamlRoot_Vtbl {
    pub base__: windows_core::IInspectable_Vtbl,
    pub Content: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut *mut core::ffi::c_void,
    ) -> windows_core::HRESULT,
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LosingFocusEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    LosingFocusEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(LosingFocusEventArgs, RoutedEventArgs);
impl LosingFocusEventArgs {
    pub fn OldFocusedElement(&self) -> windows_core::Result<DependencyObject> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).OldFocusedElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn NewFocusedElement(&self) -> windows_core::Result<DependencyObject> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).NewFocusedElement)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn FocusState(&self) -> windows_core::Result<FocusState> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).FocusState)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn TryCancel(&self) -> windows_core::Result<bool> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).TryCancel)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
}
impl windows_core::RuntimeType for LosingFocusEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, ILosingFocusEventArgs>();
}
unsafe impl windows_core::Interface for LosingFocusEventArgs {
    type Vtable = <ILosingFocusEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <ILosingFocusEventArgs as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for LosingFocusEventArgs {
    const NAME: &'static str = "Microsoft.UI.Xaml.Input.LosingFocusEventArgs";
}
unsafe impl Send for LosingFocusEventArgs {}
unsafe impl Sync for LosingFocusEventArgs {}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub X: f32,
    pub Y: f32,
    pub Width: f32,
    pub Height: f32,
}
impl windows_core::imp::TypeKind for Rect {
    type TypeKind = windows_core::imp::CopyType;
}
impl windows_core::RuntimeType for Rect {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"struct(Windows.Foundation.Rect;f4;f4;f4;f4)");
    const NAME: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::from_slice(b"Windows.Foundation.Rect");
}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutedEventArgs(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    RoutedEventArgs,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl windows_core::RuntimeType for RoutedEventArgs {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IRoutedEventArgs>();
}
unsafe impl windows_core::Interface for RoutedEventArgs {
    type Vtable = <IRoutedEventArgs as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IRoutedEventArgs as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for RoutedEventArgs {
    const NAME: &'static str = "Microsoft.UI.Xaml.RoutedEventArgs";
}
unsafe impl Send for RoutedEventArgs {}
unsafe impl Sync for RoutedEventArgs {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollView(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ScrollView,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ScrollView,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl ScrollView {
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualWidth)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualHeight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ViewportWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ViewportWidth)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ViewportHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ViewportHeight)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).XamlRoot)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).UpdateLayout)(windows_core::Interface::as_raw(
                this,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(this),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ScrollView {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IScrollView>();
}
unsafe impl windows_core::Interface for ScrollView {
    type Vtable = <IScrollView as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IScrollView as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for ScrollView {
    const NAME: &'static str = "Microsoft.UI.Xaml.Controls.ScrollView";
}
unsafe impl Send for ScrollView {}
unsafe impl Sync for ScrollView {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScrollViewer(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    ScrollViewer,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(
    ScrollViewer,
    ContentControl,
    Control,
    FrameworkElement,
    UIElement,
    DependencyObject
);
impl ScrollViewer {
    pub fn new() -> windows_core::Result<Self> {
        Self::IActivationFactory(|f| f.ActivateInstance::<Self>())
    }
    fn IActivationFactory<
        R,
        F: FnOnce(&windows_core::imp::IGenericFactory) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<
            ScrollViewer,
            windows_core::imp::IGenericFactory,
        > = windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
    pub fn ActualWidth(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualWidth)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ActualHeight(&self) -> windows_core::Result<f64> {
        let this = &windows_core::Interface::cast::<IFrameworkElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).ActualHeight)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ViewportWidth(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ViewportWidth)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn ViewportHeight(&self) -> windows_core::Result<f64> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).ViewportHeight)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .map(|| result__)
        }
    }
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).XamlRoot)(
                windows_core::Interface::as_raw(this),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).UpdateLayout)(windows_core::Interface::as_raw(
                this,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        let this = &windows_core::Interface::cast::<IUIElement>(self)?;
        unsafe {
            (windows_core::Interface::vtable(this).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(this),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for ScrollViewer {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IScrollViewer>();
}
unsafe impl windows_core::Interface for ScrollViewer {
    type Vtable = <IScrollViewer as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IScrollViewer as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for ScrollViewer {
    const NAME: &'static str = "Microsoft.UI.Xaml.Controls.ScrollViewer";
}
unsafe impl Send for ScrollViewer {}
unsafe impl Sync for ScrollViewer {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UIElement(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    UIElement,
    windows_core::IUnknown,
    windows_core::IInspectable
);
windows_core::imp::required_hierarchy!(UIElement, DependencyObject);
impl UIElement {
    pub fn XamlRoot(&self) -> windows_core::Result<XamlRoot> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).XamlRoot)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
    pub fn UpdateLayout(&self) -> windows_core::Result<()> {
        unsafe {
            (windows_core::Interface::vtable(self).UpdateLayout)(windows_core::Interface::as_raw(
                self,
            ))
            .ok()
        }
    }
    pub fn StartBringIntoViewWithOptions<P0>(&self, options: P0) -> windows_core::Result<()>
    where
        P0: windows_core::Param<BringIntoViewOptions>,
    {
        unsafe {
            (windows_core::Interface::vtable(self).StartBringIntoViewWithOptions)(
                windows_core::Interface::as_raw(self),
                options.param().abi(),
            )
            .ok()
        }
    }
}
impl windows_core::RuntimeType for UIElement {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IUIElement>();
}
unsafe impl windows_core::Interface for UIElement {
    type Vtable = <IUIElement as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IUIElement as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for UIElement {
    const NAME: &'static str = "Microsoft.UI.Xaml.UIElement";
}
unsafe impl Send for UIElement {}
unsafe impl Sync for UIElement {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VisualTreeHelper(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    VisualTreeHelper,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl VisualTreeHelper {
    pub fn GetChild<P0>(reference: P0, childindex: i32) -> windows_core::Result<DependencyObject>
    where
        P0: windows_core::Param<DependencyObject>,
    {
        Self::IVisualTreeHelperStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetChild)(
                windows_core::Interface::as_raw(this),
                reference.param().abi(),
                childindex,
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        })
    }
    pub fn GetChildrenCount<P0>(reference: P0) -> windows_core::Result<i32>
    where
        P0: windows_core::Param<DependencyObject>,
    {
        Self::IVisualTreeHelperStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetChildrenCount)(
                windows_core::Interface::as_raw(this),
                reference.param().abi(),
                &mut result__,
            )
            .map(|| result__)
        })
    }
    pub fn GetParent<P0>(reference: P0) -> windows_core::Result<DependencyObject>
    where
        P0: windows_core::Param<DependencyObject>,
    {
        Self::IVisualTreeHelperStatics(|this| unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(this).GetParent)(
                windows_core::Interface::as_raw(this),
                reference.param().abi(),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        })
    }
    fn IVisualTreeHelperStatics<
        R,
        F: FnOnce(&IVisualTreeHelperStatics) -> windows_core::Result<R>,
    >(
        callback: F,
    ) -> windows_core::Result<R> {
        static SHARED: windows_core::imp::FactoryCache<VisualTreeHelper, IVisualTreeHelperStatics> =
            windows_core::imp::FactoryCache::new();
        SHARED.call(callback)
    }
}
impl windows_core::RuntimeType for VisualTreeHelper {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IVisualTreeHelper>();
}
unsafe impl windows_core::Interface for VisualTreeHelper {
    type Vtable = <IVisualTreeHelper as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IVisualTreeHelper as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for VisualTreeHelper {
    const NAME: &'static str = "Microsoft.UI.Xaml.Media.VisualTreeHelper";
}
unsafe impl Send for VisualTreeHelper {}
unsafe impl Sync for VisualTreeHelper {}
#[repr(transparent)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XamlRoot(windows_core::IUnknown);
windows_core::imp::interface_hierarchy!(
    XamlRoot,
    windows_core::IUnknown,
    windows_core::IInspectable
);
impl XamlRoot {
    pub fn Content(&self) -> windows_core::Result<UIElement> {
        unsafe {
            let mut result__ = core::mem::zeroed();
            (windows_core::Interface::vtable(self).Content)(
                windows_core::Interface::as_raw(self),
                &mut result__,
            )
            .and_then(|| windows_core::imp::from_abi(result__))
        }
    }
}
impl windows_core::RuntimeType for XamlRoot {
    const SIGNATURE: windows_core::imp::ConstBuffer =
        windows_core::imp::ConstBuffer::for_class::<Self, IXamlRoot>();
}
unsafe impl windows_core::Interface for XamlRoot {
    type Vtable = <IXamlRoot as windows_core::Interface>::Vtable;
    const IID: windows_core::GUID = <IXamlRoot as windows_core::Interface>::IID;
    const INSPECTABLE: bool = true;
}
impl windows_core::RuntimeName for XamlRoot {
    const NAME: &'static str = "Microsoft.UI.Xaml.XamlRoot";
}
unsafe impl Send for XamlRoot {}
unsafe impl Sync for XamlRoot {}
