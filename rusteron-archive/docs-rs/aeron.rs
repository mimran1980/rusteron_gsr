type aeron_client_registering_resource_t = aeron_client_registering_resource_stct;
#[derive(Clone)]
pub struct DarwinPthreadHandlerRec {
    inner: CResource<__darwin_pthread_handler_rec>,
}
impl core::fmt::Debug for DarwinPthreadHandlerRec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(DarwinPthreadHandlerRec))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(DarwinPthreadHandlerRec))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl DarwinPthreadHandlerRec {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(__darwin_pthread_handler_rec)
                );
                let inst: __darwin_pthread_handler_rec = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut __darwin_pthread_handler_rec = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(__darwin_pthread_handler_rec)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut __darwin_pthread_handler_rec {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut __darwin_pthread_handler_rec {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &__darwin_pthread_handler_rec {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for DarwinPthreadHandlerRec {
    type Target = __darwin_pthread_handler_rec;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut __darwin_pthread_handler_rec> for DarwinPthreadHandlerRec {
    #[inline]
    fn from(value: *mut __darwin_pthread_handler_rec) -> Self {
        DarwinPthreadHandlerRec {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<DarwinPthreadHandlerRec> for *mut __darwin_pthread_handler_rec {
    #[inline]
    fn from(value: DarwinPthreadHandlerRec) -> Self {
        value.get_inner()
    }
}
impl From<&DarwinPthreadHandlerRec> for *mut __darwin_pthread_handler_rec {
    #[inline]
    fn from(value: &DarwinPthreadHandlerRec) -> Self {
        value.get_inner()
    }
}
impl From<DarwinPthreadHandlerRec> for __darwin_pthread_handler_rec {
    #[inline]
    fn from(value: DarwinPthreadHandlerRec) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const __darwin_pthread_handler_rec> for DarwinPthreadHandlerRec {
    #[inline]
    fn from(value: *const __darwin_pthread_handler_rec) -> Self {
        DarwinPthreadHandlerRec {
            inner: CResource::Borrowed(value as *mut __darwin_pthread_handler_rec),
        }
    }
}
impl From<__darwin_pthread_handler_rec> for DarwinPthreadHandlerRec {
    #[inline]
    fn from(value: __darwin_pthread_handler_rec) -> Self {
        DarwinPthreadHandlerRec {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
use crate::AeronErrorType::Unknown;
#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::cell::UnsafeCell;
use std::fmt::Formatter;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut};
pub enum CResource<T> {
    OwnedOnHeap(std::rc::Rc<ManagedCResource<T>>),
    #[doc = " stored on stack, unsafe, use with care"]
    OwnedOnStack(std::mem::MaybeUninit<T>),
    Borrowed(*mut T),
}
impl<T: Clone> Clone for CResource<T> {
    fn clone(&self) -> Self {
        unsafe {
            match self {
                CResource::OwnedOnHeap(r) => CResource::OwnedOnHeap(r.clone()),
                CResource::OwnedOnStack(r) => {
                    CResource::OwnedOnStack(MaybeUninit::new(r.assume_init_ref().clone()))
                }
                CResource::Borrowed(r) => CResource::Borrowed(r.clone()),
            }
        }
    }
}
impl<T> CResource<T> {
    #[inline]
    pub fn get(&self) -> *mut T {
        match self {
            CResource::OwnedOnHeap(r) => r.get(),
            CResource::OwnedOnStack(r) => r.as_ptr() as *mut T,
            CResource::Borrowed(r) => *r,
        }
    }
    #[inline]
    pub fn add_dependency<D: std::any::Any>(&self, dep: D) {
        match self {
            CResource::OwnedOnHeap(r) => r.add_dependency(dep),
            CResource::OwnedOnStack(_) | CResource::Borrowed(_) => {
                unreachable!("only owned on heap")
            }
        }
    }
    #[inline]
    pub fn get_dependency<V: Clone + 'static>(&self) -> Option<V> {
        match self {
            CResource::OwnedOnHeap(r) => r.get_dependency(),
            CResource::OwnedOnStack(_) | CResource::Borrowed(_) => None,
        }
    }
    #[inline]
    pub fn as_owned(&self) -> Option<&std::rc::Rc<ManagedCResource<T>>> {
        match self {
            CResource::OwnedOnHeap(r) => Some(r),
            CResource::OwnedOnStack(_) | CResource::Borrowed(_) => None,
        }
    }
}
impl<T> std::fmt::Debug for CResource<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = std::any::type_name::<T>();
        match self {
            CResource::OwnedOnHeap(r) => {
                write!(f, "{name} heap({:?})", r)
            }
            CResource::OwnedOnStack(r) => {
                write!(f, "{name} stack({:?})", *r)
            }
            CResource::Borrowed(r) => {
                write!(f, "{name} borrowed ({:?})", r)
            }
        }
    }
}
#[doc = " A custom struct for managing C resources with automatic cleanup."]
#[doc = ""]
#[doc = " It handles initialisation and clean-up of the resource and ensures that resources"]
#[doc = " are properly released when they go out of scope."]
#[allow(dead_code)]
pub struct ManagedCResource<T> {
    resource: *mut T,
    cleanup: Option<Box<dyn FnMut(*mut *mut T) -> i32>>,
    cleanup_struct: bool,
    #[doc = " if someone externally rusteron calls close"]
    close_already_called: std::cell::Cell<bool>,
    #[doc = " if there is a c method to verify it someone has closed it, only few structs have this functionality"]
    check_for_is_closed: Option<fn(*mut T) -> bool>,
    #[doc = " this will be called if closed hasn't already happened even if its borrowed"]
    auto_close: std::cell::Cell<bool>,
    #[doc = " to prevent the dependencies from being dropped as you have a copy here,"]
    #[doc = " for example, you want to have a dependency to aeron for any async jobs so aeron doesnt get dropped first"]
    #[doc = " when you have a publication/subscription"]
    #[doc = " Note empty vec does not allocate on heap"]
    dependencies: UnsafeCell<Vec<std::rc::Rc<dyn std::any::Any>>>,
}
impl<T> std::fmt::Debug for ManagedCResource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("ManagedCResource");
        if !self.close_already_called.get()
            && !self.resource.is_null()
            && !self
                .check_for_is_closed
                .as_ref()
                .map_or(false, |f| f(self.resource))
        {
            debug_struct.field("resource", &self.resource);
        }
        debug_struct
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}
impl<T> ManagedCResource<T> {
    #[doc = " Creates a new ManagedCResource with a given initializer and cleanup function."]
    #[doc = ""]
    #[doc = " The initializer is a closure that attempts to initialize the resource."]
    #[doc = " If initialization fails, the initializer should return an error code."]
    #[doc = " The cleanup function is used to release the resource when it's no longer needed."]
    #[doc = " `cleanup_struct` where it should clean up the struct in rust"]
    pub fn new(
        init: impl FnOnce(*mut *mut T) -> i32,
        cleanup: Option<Box<dyn FnMut(*mut *mut T) -> i32>>,
        cleanup_struct: bool,
        check_for_is_closed: Option<fn(*mut T) -> bool>,
    ) -> Result<Self, AeronCError> {
        let resource = Self::initialise(init)?;
        let result = Self {
            resource,
            cleanup,
            cleanup_struct,
            close_already_called: std::cell::Cell::new(false),
            check_for_is_closed,
            auto_close: std::cell::Cell::new(false),
            dependencies: UnsafeCell::new(vec![]),
        };
        #[cfg(feature = "extra-logging")]
        log::info!("created c resource: {:?}", result);
        Ok(result)
    }
    pub fn initialise(
        init: impl FnOnce(*mut *mut T) -> i32 + Sized,
    ) -> Result<*mut T, AeronCError> {
        let mut resource: *mut T = std::ptr::null_mut();
        let result = init(&mut resource);
        if result < 0 || resource.is_null() {
            return Err(AeronCError::from_code(result));
        }
        Ok(resource)
    }
    pub fn is_closed_already_called(&self) -> bool {
        self.close_already_called.get()
            || self.resource.is_null()
            || self
                .check_for_is_closed
                .as_ref()
                .map_or(false, |f| f(self.resource))
    }
    #[doc = " Gets a raw pointer to the resource."]
    #[inline(always)]
    pub fn get(&self) -> *mut T {
        self.resource
    }
    #[inline(always)]
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.resource }
    }
    #[inline]
    pub fn add_dependency<D: std::any::Any>(&self, dep: D) {
        if let Some(dep) =
            (&dep as &dyn std::any::Any).downcast_ref::<std::rc::Rc<dyn std::any::Any>>()
        {
            unsafe {
                (*self.dependencies.get()).push(dep.clone());
            }
        } else {
            unsafe {
                (*self.dependencies.get()).push(std::rc::Rc::new(dep));
            }
        }
    }
    #[inline]
    pub fn get_dependency<V: Clone + 'static>(&self) -> Option<V> {
        unsafe {
            (*self.dependencies.get())
                .iter()
                .filter_map(|x| x.as_ref().downcast_ref::<V>().cloned())
                .next()
        }
    }
    #[doc = " Closes the resource by calling the cleanup function."]
    #[doc = ""]
    #[doc = " If cleanup fails, it returns an `AeronError`."]
    pub fn close(&mut self) -> Result<(), AeronCError> {
        if self.close_already_called.get() {
            return Ok(());
        }
        self.close_already_called.set(true);
        let already_closed = self
            .check_for_is_closed
            .as_ref()
            .map_or(false, |f| f(self.resource));
        if let Some(mut cleanup) = self.cleanup.take() {
            if !self.resource.is_null() {
                if !already_closed {
                    let result = cleanup(&mut self.resource);
                    if result < 0 {
                        return Err(AeronCError::from_code(result));
                    }
                }
                self.resource = std::ptr::null_mut();
            }
        }
        Ok(())
    }
}
impl<T> Drop for ManagedCResource<T> {
    fn drop(&mut self) {
        if !self.resource.is_null() {
            let already_closed = self.close_already_called.get()
                || self
                    .check_for_is_closed
                    .as_ref()
                    .map_or(false, |f| f(self.resource));
            let resource = if already_closed {
                self.resource
            } else {
                self.resource.clone()
            };
            if !already_closed {
                #[cfg(feature = "extra-logging")]
                log::info!("closing c resource: {:?}", self);
                let _ = self.close();
            }
            self.close_already_called.set(true);
            if self.cleanup_struct {
                #[cfg(feature = "extra-logging")]
                log::info!("closing rust struct resource: {:?}", resource);
                unsafe {
                    let _ = Box::from_raw(resource);
                }
            }
        }
    }
}
#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub enum AeronErrorType {
    GenericError,
    ClientErrorDriverTimeout,
    ClientErrorClientTimeout,
    ClientErrorConductorServiceTimeout,
    ClientErrorBufferFull,
    PublicationBackPressured,
    PublicationAdminAction,
    PublicationClosed,
    PublicationMaxPositionExceeded,
    PublicationError,
    TimedOut,
    Unknown(i32),
}
impl From<AeronErrorType> for AeronCError {
    fn from(value: AeronErrorType) -> Self {
        AeronCError::from_code(value.code())
    }
}
impl AeronErrorType {
    pub fn code(&self) -> i32 {
        match self {
            AeronErrorType::GenericError => -1,
            AeronErrorType::ClientErrorDriverTimeout => -1000,
            AeronErrorType::ClientErrorClientTimeout => -1001,
            AeronErrorType::ClientErrorConductorServiceTimeout => -1002,
            AeronErrorType::ClientErrorBufferFull => -1003,
            AeronErrorType::PublicationBackPressured => -2,
            AeronErrorType::PublicationAdminAction => -3,
            AeronErrorType::PublicationClosed => -4,
            AeronErrorType::PublicationMaxPositionExceeded => -5,
            AeronErrorType::PublicationError => -6,
            AeronErrorType::TimedOut => -234324,
            AeronErrorType::Unknown(code) => *code,
        }
    }
    pub fn is_back_pressured(&self) -> bool {
        self == &AeronErrorType::PublicationBackPressured
    }
    pub fn is_admin_action(&self) -> bool {
        self == &AeronErrorType::PublicationAdminAction
    }
    pub fn is_back_pressured_or_admin_action(&self) -> bool {
        self.is_back_pressured() || self.is_admin_action()
    }
    pub fn from_code(code: i32) -> Self {
        match code {
            -1 => AeronErrorType::GenericError,
            -1000 => AeronErrorType::ClientErrorDriverTimeout,
            -1001 => AeronErrorType::ClientErrorClientTimeout,
            -1002 => AeronErrorType::ClientErrorConductorServiceTimeout,
            -1003 => AeronErrorType::ClientErrorBufferFull,
            -2 => AeronErrorType::PublicationBackPressured,
            -3 => AeronErrorType::PublicationAdminAction,
            -4 => AeronErrorType::PublicationClosed,
            -5 => AeronErrorType::PublicationMaxPositionExceeded,
            -6 => AeronErrorType::PublicationError,
            -234324 => AeronErrorType::TimedOut,
            _ => Unknown(code),
        }
    }
    pub fn to_string(&self) -> &'static str {
        match self {
            AeronErrorType::GenericError => "Generic Error",
            AeronErrorType::ClientErrorDriverTimeout => "Client Error Driver Timeout",
            AeronErrorType::ClientErrorClientTimeout => "Client Error Client Timeout",
            AeronErrorType::ClientErrorConductorServiceTimeout => {
                "Client Error Conductor Service Timeout"
            }
            AeronErrorType::ClientErrorBufferFull => "Client Error Buffer Full",
            AeronErrorType::PublicationBackPressured => "Publication Back Pressured",
            AeronErrorType::PublicationAdminAction => "Publication Admin Action",
            AeronErrorType::PublicationClosed => "Publication Closed",
            AeronErrorType::PublicationMaxPositionExceeded => "Publication Max Position Exceeded",
            AeronErrorType::PublicationError => "Publication Error",
            AeronErrorType::TimedOut => "Timed Out",
            AeronErrorType::Unknown(_) => "Unknown Error",
        }
    }
}
#[doc = " Represents an Aeron-specific error with a code and an optional message."]
#[doc = ""]
#[doc = " The error code is derived from Aeron C API calls."]
#[doc = " Use `get_last_err_message()` to retrieve the last human-readable message, if available."]
#[derive(Eq, PartialEq, Clone)]
pub struct AeronCError {
    pub code: i32,
}
impl AeronCError {
    #[doc = " Creates an AeronError from the error code returned by Aeron."]
    #[doc = ""]
    #[doc = " Error codes below zero are considered failure."]
    pub fn from_code(code: i32) -> Self {
        #[cfg(feature = "backtrace")]
        {
            if code < 0 {
                let backtrace = Backtrace::capture();
                let backtrace = format!("{:?}", backtrace);
                let re =
                    regex::Regex::new(r#"fn: "([^"]+)", file: "([^"]+)", line: (\d+)"#).unwrap();
                let mut lines = String::new();
                re.captures_iter(&backtrace).for_each(|cap| {
                    let function = &cap[1];
                    let mut file = cap[2].to_string();
                    let line = &cap[3];
                    if file.starts_with("./") {
                        file = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), &file[2..]);
                    } else if file.starts_with("/rustc/") {
                        file = file.split("/").last().unwrap().to_string();
                    }
                    lines.push_str(&format!(" {file}:{line} in {function}\n"));
                });
                log::error!(
                    "Aeron C error code: {}, kind: '{:?}'\n{}",
                    code,
                    AeronErrorType::from_code(code),
                    lines
                );
            }
        }
        AeronCError { code }
    }
    pub fn kind(&self) -> AeronErrorType {
        AeronErrorType::from_code(self.code)
    }
    pub fn is_back_pressured(&self) -> bool {
        self.kind().is_back_pressured()
    }
    pub fn is_admin_action(&self) -> bool {
        self.kind().is_admin_action()
    }
    pub fn is_back_pressured_or_admin_action(&self) -> bool {
        self.kind().is_back_pressured_or_admin_action()
    }
}
#[doc = " # Handler"]
#[doc = ""]
#[doc = " `Handler` is a struct that wraps a raw pointer and a drop flag."]
#[doc = ""]
#[doc = " **Important:** `Handler` *MAY* not get dropped automatically. It depends if aeron takes ownership."]
#[doc = " For example for global level handlers e.g. error handler aeron will release this handle when closing."]
#[doc = ""]
#[doc = " You need to call the `release` method if you want to clear the memory manually."]
#[doc = " Its important that you test this out as aeron may do it when closing aeron client."]
#[doc = ""]
#[doc = " ## Example"]
#[doc = ""]
#[doc = " ```no_compile"]
#[doc = " use rusteron_code_gen::Handler;"]
#[doc = " let handler = Handler::leak(your_value);"]
#[doc = " // When you are done with the handler"]
#[doc = " handler.release();"]
#[doc = " ```"]
pub struct Handler<T> {
    raw_ptr: *mut T,
    should_drop: bool,
}
unsafe impl<T> Send for Handler<T> {}
unsafe impl<T> Sync for Handler<T> {}
#[doc = " Utility method for setting empty handlers"]
pub struct Handlers;
impl<T> Handler<T> {
    pub fn leak(handler: T) -> Self {
        let raw_ptr = Box::into_raw(Box::new(handler)) as *mut _;
        #[cfg(feature = "extra-logging")]
        log::info!("creating handler {:?}", raw_ptr);
        Self {
            raw_ptr,
            should_drop: true,
        }
    }
    pub fn is_none(&self) -> bool {
        self.raw_ptr.is_null()
    }
    pub fn as_raw(&self) -> *mut std::os::raw::c_void {
        self.raw_ptr as *mut std::os::raw::c_void
    }
    pub fn release(&mut self) {
        if self.should_drop && !self.raw_ptr.is_null() {
            unsafe {
                #[cfg(feature = "extra-logging")]
                log::info!("dropping handler {:?}", self.raw_ptr);
                let _ = Box::from_raw(self.raw_ptr as *mut T);
                self.should_drop = false;
            }
        }
    }
    pub unsafe fn new(raw_ptr: *mut T, should_drop: bool) -> Self {
        Self {
            raw_ptr,
            should_drop,
        }
    }
}
impl<T> Deref for Handler<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.raw_ptr as &T }
    }
}
impl<T> DerefMut for Handler<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.raw_ptr as &mut T }
    }
}
pub fn find_unused_udp_port(start_port: u16) -> Option<u16> {
    let end_port = u16::MAX;
    for port in start_port..=end_port {
        if is_udp_port_available(port) {
            return Some(port);
        }
    }
    None
}
pub fn is_udp_port_available(port: u16) -> bool {
    std::net::UdpSocket::bind(("127.0.0.1", port)).is_ok()
}
#[doc = " Represents the Aeron URI parser and handler."]
pub struct ChannelUri {}
impl ChannelUri {
    pub const AERON_SCHEME: &'static str = "aeron";
    pub const SPY_QUALIFIER: &'static str = "aeron-spy";
    pub const MAX_URI_LENGTH: usize = 4095;
}
pub const DRIVER_TIMEOUT_MS_DEFAULT: u64 = 10_000;
pub const AERON_DIR_PROP_NAME: &str = "aeron.dir";
pub const AERON_IPC_MEDIA: &str = "aeron:ipc";
pub const AERON_UDP_MEDIA: &str = "aeron:udp";
pub const SPY_PREFIX: &str = "aeron-spy:";
pub const TAG_PREFIX: &str = "tag:";
#[doc = " Enum for media types."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Media {
    Ipc,
    Udp,
}
impl Media {
    pub fn as_str(&self) -> &'static str {
        match self {
            Media::Ipc => "ipc",
            Media::Udp => "udp",
        }
    }
}
#[doc = " Enum for control modes."]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlMode {
    Manual,
    Dynamic,
    #[doc = " this is a beta feature useful when dealing with docker containers and networking"]
    Response,
}
impl ControlMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ControlMode::Manual => "manual",
            ControlMode::Dynamic => "dynamic",
            ControlMode::Response => "response",
        }
    }
}
#[cfg(test)]
#[allow(dead_code)]
pub(crate) mod test_alloc {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicIsize, Ordering};
    #[doc = " A simple global allocator that tracks the net allocation count."]
    #[doc = " For very simple examples can do allocation count before and after your test."]
    #[doc = " This does not work well with logger, running media driver, etc. Only for the most"]
    #[doc = " basic controlled examples"]
    pub struct CountingAllocator {
        allocs: AtomicIsize,
    }
    impl CountingAllocator {
        pub const fn new() -> Self {
            Self {
                allocs: AtomicIsize::new(0),
            }
        }
        #[doc = " Returns the current allocation counter value."]
        fn current(&self) -> isize {
            self.allocs.load(Ordering::SeqCst)
        }
    }
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            self.allocs.fetch_add(1, Ordering::SeqCst);
            System.alloc(layout)
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            self.allocs.fetch_sub(1, Ordering::SeqCst);
            System.dealloc(ptr, layout)
        }
    }
    #[global_allocator]
    static GLOBAL: CountingAllocator = CountingAllocator::new();
    #[doc = " Returns the current allocation counter value."]
    pub fn current_allocs() -> isize {
        GLOBAL.current()
    }
}
pub trait IntoCString {
    fn into_c_string(self) -> std::ffi::CString;
}
impl IntoCString for std::ffi::CString {
    fn into_c_string(self) -> std::ffi::CString {
        self
    }
}
impl IntoCString for &str {
    fn into_c_string(self) -> std::ffi::CString {
        #[cfg(feature = "extra-logging")]
        log::info!("created c string on heap: {:?}", self);
        std::ffi::CString::new(self).expect("failed to create CString")
    }
}
impl IntoCString for String {
    fn into_c_string(self) -> std::ffi::CString {
        #[cfg(feature = "extra-logging")]
        log::info!("created c string on heap: {:?}", self);
        std::ffi::CString::new(self).expect("failed to create CString")
    }
}
#[derive(Clone)]
pub struct OpaquePthreadAttr {
    inner: CResource<_opaque_pthread_attr_t>,
}
impl core::fmt::Debug for OpaquePthreadAttr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(OpaquePthreadAttr))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(OpaquePthreadAttr))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl OpaquePthreadAttr {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(_opaque_pthread_attr_t)
                );
                let inst: _opaque_pthread_attr_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut _opaque_pthread_attr_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(_opaque_pthread_attr_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut _opaque_pthread_attr_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut _opaque_pthread_attr_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &_opaque_pthread_attr_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for OpaquePthreadAttr {
    type Target = _opaque_pthread_attr_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut _opaque_pthread_attr_t> for OpaquePthreadAttr {
    #[inline]
    fn from(value: *mut _opaque_pthread_attr_t) -> Self {
        OpaquePthreadAttr {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<OpaquePthreadAttr> for *mut _opaque_pthread_attr_t {
    #[inline]
    fn from(value: OpaquePthreadAttr) -> Self {
        value.get_inner()
    }
}
impl From<&OpaquePthreadAttr> for *mut _opaque_pthread_attr_t {
    #[inline]
    fn from(value: &OpaquePthreadAttr) -> Self {
        value.get_inner()
    }
}
impl From<OpaquePthreadAttr> for _opaque_pthread_attr_t {
    #[inline]
    fn from(value: OpaquePthreadAttr) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const _opaque_pthread_attr_t> for OpaquePthreadAttr {
    #[inline]
    fn from(value: *const _opaque_pthread_attr_t) -> Self {
        OpaquePthreadAttr {
            inner: CResource::Borrowed(value as *mut _opaque_pthread_attr_t),
        }
    }
}
impl From<_opaque_pthread_attr_t> for OpaquePthreadAttr {
    #[inline]
    fn from(value: _opaque_pthread_attr_t) -> Self {
        OpaquePthreadAttr {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct OpaquePthreadCond {
    inner: CResource<_opaque_pthread_cond_t>,
}
impl core::fmt::Debug for OpaquePthreadCond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(OpaquePthreadCond))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(OpaquePthreadCond))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl OpaquePthreadCond {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(_opaque_pthread_cond_t)
                );
                let inst: _opaque_pthread_cond_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut _opaque_pthread_cond_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(_opaque_pthread_cond_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut _opaque_pthread_cond_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut _opaque_pthread_cond_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &_opaque_pthread_cond_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for OpaquePthreadCond {
    type Target = _opaque_pthread_cond_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut _opaque_pthread_cond_t> for OpaquePthreadCond {
    #[inline]
    fn from(value: *mut _opaque_pthread_cond_t) -> Self {
        OpaquePthreadCond {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<OpaquePthreadCond> for *mut _opaque_pthread_cond_t {
    #[inline]
    fn from(value: OpaquePthreadCond) -> Self {
        value.get_inner()
    }
}
impl From<&OpaquePthreadCond> for *mut _opaque_pthread_cond_t {
    #[inline]
    fn from(value: &OpaquePthreadCond) -> Self {
        value.get_inner()
    }
}
impl From<OpaquePthreadCond> for _opaque_pthread_cond_t {
    #[inline]
    fn from(value: OpaquePthreadCond) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const _opaque_pthread_cond_t> for OpaquePthreadCond {
    #[inline]
    fn from(value: *const _opaque_pthread_cond_t) -> Self {
        OpaquePthreadCond {
            inner: CResource::Borrowed(value as *mut _opaque_pthread_cond_t),
        }
    }
}
impl From<_opaque_pthread_cond_t> for OpaquePthreadCond {
    #[inline]
    fn from(value: _opaque_pthread_cond_t) -> Self {
        OpaquePthreadCond {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct OpaquePthreadMutex {
    inner: CResource<_opaque_pthread_mutex_t>,
}
impl core::fmt::Debug for OpaquePthreadMutex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(OpaquePthreadMutex))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(OpaquePthreadMutex))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl OpaquePthreadMutex {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(_opaque_pthread_mutex_t)
                );
                let inst: _opaque_pthread_mutex_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut _opaque_pthread_mutex_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(_opaque_pthread_mutex_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut _opaque_pthread_mutex_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut _opaque_pthread_mutex_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &_opaque_pthread_mutex_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for OpaquePthreadMutex {
    type Target = _opaque_pthread_mutex_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut _opaque_pthread_mutex_t> for OpaquePthreadMutex {
    #[inline]
    fn from(value: *mut _opaque_pthread_mutex_t) -> Self {
        OpaquePthreadMutex {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<OpaquePthreadMutex> for *mut _opaque_pthread_mutex_t {
    #[inline]
    fn from(value: OpaquePthreadMutex) -> Self {
        value.get_inner()
    }
}
impl From<&OpaquePthreadMutex> for *mut _opaque_pthread_mutex_t {
    #[inline]
    fn from(value: &OpaquePthreadMutex) -> Self {
        value.get_inner()
    }
}
impl From<OpaquePthreadMutex> for _opaque_pthread_mutex_t {
    #[inline]
    fn from(value: OpaquePthreadMutex) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const _opaque_pthread_mutex_t> for OpaquePthreadMutex {
    #[inline]
    fn from(value: *const _opaque_pthread_mutex_t) -> Self {
        OpaquePthreadMutex {
            inner: CResource::Borrowed(value as *mut _opaque_pthread_mutex_t),
        }
    }
}
impl From<_opaque_pthread_mutex_t> for OpaquePthreadMutex {
    #[inline]
    fn from(value: _opaque_pthread_mutex_t) -> Self {
        OpaquePthreadMutex {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct OpaquePthread {
    inner: CResource<_opaque_pthread_t>,
}
impl core::fmt::Debug for OpaquePthread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(OpaquePthread))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(OpaquePthread))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl OpaquePthread {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(_opaque_pthread_t)
                );
                let inst: _opaque_pthread_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut _opaque_pthread_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(_opaque_pthread_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut _opaque_pthread_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut _opaque_pthread_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &_opaque_pthread_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for OpaquePthread {
    type Target = _opaque_pthread_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut _opaque_pthread_t> for OpaquePthread {
    #[inline]
    fn from(value: *mut _opaque_pthread_t) -> Self {
        OpaquePthread {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<OpaquePthread> for *mut _opaque_pthread_t {
    #[inline]
    fn from(value: OpaquePthread) -> Self {
        value.get_inner()
    }
}
impl From<&OpaquePthread> for *mut _opaque_pthread_t {
    #[inline]
    fn from(value: &OpaquePthread) -> Self {
        value.get_inner()
    }
}
impl From<OpaquePthread> for _opaque_pthread_t {
    #[inline]
    fn from(value: OpaquePthread) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const _opaque_pthread_t> for OpaquePthread {
    #[inline]
    fn from(value: *const _opaque_pthread_t) -> Self {
        OpaquePthread {
            inner: CResource::Borrowed(value as *mut _opaque_pthread_t),
        }
    }
}
impl From<_opaque_pthread_t> for OpaquePthread {
    #[inline]
    fn from(value: _opaque_pthread_t) -> Self {
        OpaquePthread {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronArchiveAsyncConnect {
    inner: CResource<aeron_archive_async_connect_t>,
}
impl core::fmt::Debug for AeronArchiveAsyncConnect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveAsyncConnect))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveAsyncConnect))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronArchiveAsyncConnect {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_async_connect_t)
                );
                let inst: aeron_archive_async_connect_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_async_connect_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_async_connect_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_async_connect_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_async_connect_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_async_connect_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveAsyncConnect {
    type Target = aeron_archive_async_connect_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_async_connect_t> for AeronArchiveAsyncConnect {
    #[inline]
    fn from(value: *mut aeron_archive_async_connect_t) -> Self {
        AeronArchiveAsyncConnect {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveAsyncConnect> for *mut aeron_archive_async_connect_t {
    #[inline]
    fn from(value: AeronArchiveAsyncConnect) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveAsyncConnect> for *mut aeron_archive_async_connect_t {
    #[inline]
    fn from(value: &AeronArchiveAsyncConnect) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveAsyncConnect> for aeron_archive_async_connect_t {
    #[inline]
    fn from(value: AeronArchiveAsyncConnect) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_async_connect_t> for AeronArchiveAsyncConnect {
    #[inline]
    fn from(value: *const aeron_archive_async_connect_t) -> Self {
        AeronArchiveAsyncConnect {
            inner: CResource::Borrowed(value as *mut aeron_archive_async_connect_t),
        }
    }
}
impl From<aeron_archive_async_connect_t> for AeronArchiveAsyncConnect {
    #[inline]
    fn from(value: aeron_archive_async_connect_t) -> Self {
        AeronArchiveAsyncConnect {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl AeronArchive {
    #[inline]
    pub fn new(async_: &AeronArchiveAsyncConnect) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_archive_async_connect_poll(ctx_field, async_.into()) },
            None,
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        })
    }
}
impl AeronArchiveContext {
    #[inline]
    pub fn aeron_archive_async_connect(&self) -> Result<AeronArchiveAsyncConnect, AeronCError> {
        let mut result = AeronArchiveAsyncConnect::new(self);
        if let Ok(result) = &mut result {
            result.inner.add_dependency(self.clone());
        }
        result
    }
}
impl AeronArchiveContext {
    #[inline]
    pub fn aeron_archive_connect(
        &self,
        timeout: std::time::Duration,
    ) -> Result<AeronArchive, AeronCError> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(poller) = AeronArchiveAsyncConnect::new(self) {
                while start.elapsed() <= timeout {
                    if let Some(result) = poller.poll()? {
                        return Ok(result);
                    }
                    #[cfg(debug_assertions)]
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            if start.elapsed() > timeout {
                log::error!("failed async poll for {:?}", self);
                return Err(AeronErrorType::TimedOut.into());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
impl AeronArchiveAsyncConnect {
    #[inline]
    pub fn new(ctx: &AeronArchiveContext) -> Result<Self, AeronCError> {
        let resource_async = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_archive_async_connect(ctx_field, ctx.into()) },
            None,
            false,
            None,
        )?;
        let result = Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_async)),
        };
        result.inner.add_dependency(ctx.clone());
        Ok(result)
    }
    pub fn poll(&self) -> Result<Option<AeronArchive>, AeronCError> {
        let mut result = AeronArchive::new(self);
        if let Ok(result) = &mut result {
            unsafe {
                for d in (&mut *self.inner.as_owned().unwrap().dependencies.get()).iter_mut() {
                    result.inner.add_dependency(d.clone());
                }
                result.inner.as_owned().unwrap().auto_close.set(true);
            }
        }
        match result {
            Ok(result) => Ok(Some(result)),
            Err(AeronCError { code }) if code == 0 => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_blocking(&self, timeout: std::time::Duration) -> Result<AeronArchive, AeronCError> {
        if let Some(result) = self.poll()? {
            return Ok(result);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Some(result) = self.poll()? {
                return Ok(result);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {:?}", self);
        Err(AeronErrorType::TimedOut.into())
    }
}
#[derive(Clone)]
pub struct AeronArchiveContext {
    inner: CResource<aeron_archive_context_t>,
}
impl core::fmt::Debug for AeronArchiveContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveContext))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveContext))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronArchiveContext {
    #[doc = "Create an `AeronArchiveContext` struct."]
    #[doc = ""]
    pub fn new() -> Result<Self, AeronCError> {
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_archive_context_init(ctx_field) },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_archive_context_close(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Close and delete the `AeronArchiveContext` struct."]
    #[doc = ""]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_context_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the client used for communicating with the local Media Driver."]
    #[doc = " \n"]
    #[doc = " This client will be closed with the `AeronArchive` is closed if aeron_archive_context_set_owns_aeron_client is true."]
    pub fn set_aeron(&self, aeron: &Aeron) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_aeron(self.get_inner(), aeron.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_aeron(&self) -> Aeron {
        unsafe {
            let result = aeron_archive_context_get_aeron(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify whether or not this context owns the client and, therefore, takes responsibility for closing it."]
    pub fn set_owns_aeron_client(&self, owns_aeron_client: bool) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_owns_aeron_client(
                self.get_inner(),
                owns_aeron_client.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_owns_aeron_client(&self) -> bool {
        unsafe {
            let result = aeron_archive_context_get_owns_aeron_client(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the top level Aeron directory used for communication between the Aeron client and the Media Driver."]
    pub fn set_aeron_directory_name(
        &self,
        aeron_directory_name: &std::ffi::CStr,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_aeron_directory_name(
                self.get_inner(),
                aeron_directory_name.as_ptr(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_aeron_directory_name(&self) -> &str {
        unsafe {
            let result = aeron_archive_context_get_aeron_directory_name(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Specify the channel used for sending requests to the Aeron Archive."]
    pub fn set_control_request_channel(
        &self,
        control_request_channel: &std::ffi::CStr,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_request_channel(
                self.get_inner(),
                control_request_channel.as_ptr(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_request_channel(&self) -> &str {
        unsafe {
            let result = aeron_archive_context_get_control_request_channel(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Specify the stream used for sending requests to the Aeron Archive."]
    pub fn set_control_request_stream_id(
        &self,
        control_request_stream_id: i32,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_request_stream_id(
                self.get_inner(),
                control_request_stream_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_request_stream_id(&self) -> i32 {
        unsafe {
            let result = aeron_archive_context_get_control_request_stream_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the channel used for receiving responses from the Aeron Archive."]
    pub fn set_control_response_channel(
        &self,
        control_response_channel: &std::ffi::CStr,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_response_channel(
                self.get_inner(),
                control_response_channel.as_ptr(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_response_channel(&self) -> &str {
        unsafe {
            let result = aeron_archive_context_get_control_response_channel(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Specify the stream used for receiving responses from the Aeron Archive."]
    pub fn set_control_response_stream_id(
        &self,
        control_response_stream_id: i32,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_response_stream_id(
                self.get_inner(),
                control_response_stream_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_response_stream_id(&self) -> i32 {
        unsafe {
            let result = aeron_archive_context_get_control_response_stream_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the channel used for receiving recording events from the Aeron Archive."]
    pub fn set_recording_events_channel(
        &self,
        recording_events_channel: &std::ffi::CStr,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_recording_events_channel(
                self.get_inner(),
                recording_events_channel.as_ptr(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_recording_events_channel(&self) -> &str {
        unsafe {
            let result = aeron_archive_context_get_recording_events_channel(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Specify the stream id used for recording events channel."]
    pub fn set_recording_events_stream_id(
        &self,
        recording_events_stream_id: i32,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_recording_events_stream_id(
                self.get_inner(),
                recording_events_stream_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_recording_events_stream_id(&self) -> i32 {
        unsafe {
            let result = aeron_archive_context_get_recording_events_stream_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the message timeout, in nanoseconds, to wait for sending or receiving a message."]
    pub fn set_message_timeout_ns(&self, message_timeout_ns: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_message_timeout_ns(
                self.get_inner(),
                message_timeout_ns.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_message_timeout_ns(&self) -> u64 {
        unsafe {
            let result = aeron_archive_context_get_message_timeout_ns(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the default term buffer length for the control request/response channels."]
    pub fn set_control_term_buffer_length(
        &self,
        control_term_buffer_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_term_buffer_length(
                self.get_inner(),
                control_term_buffer_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_term_buffer_length(&self) -> usize {
        unsafe {
            let result = aeron_archive_context_get_control_term_buffer_length(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the default MTU length for the control request/response channels."]
    pub fn set_control_mtu_length(&self, control_mtu_length: usize) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_mtu_length(
                self.get_inner(),
                control_mtu_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_mtu_length(&self) -> usize {
        unsafe {
            let result = aeron_archive_context_get_control_mtu_length(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the default MTU length for the control request/response channels."]
    pub fn set_control_term_buffer_sparse(
        &self,
        control_term_buffer_sparse: bool,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_control_term_buffer_sparse(
                self.get_inner(),
                control_term_buffer_sparse.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_control_term_buffer_sparse(&self) -> bool {
        unsafe {
            let result = aeron_archive_context_get_control_term_buffer_sparse(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Specify the idle strategy function and associated state used by the client between polling calls."]
    pub fn set_idle_strategy<AeronIdleStrategyFuncHandlerImpl: AeronIdleStrategyFuncCallback>(
        &self,
        idle_strategy_func: Option<&Handler<AeronIdleStrategyFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_idle_strategy(
                self.get_inner(),
                {
                    let callback: aeron_idle_strategy_func_t = if idle_strategy_func.is_none() {
                        None
                    } else {
                        Some(
                            aeron_idle_strategy_func_t_callback::<AeronIdleStrategyFuncHandlerImpl>,
                        )
                    };
                    callback
                },
                idle_strategy_func
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the idle strategy function and associated state used by the client between polling calls."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_idle_strategy_once<
        AeronIdleStrategyFuncHandlerImpl: FnMut(::std::os::raw::c_int) -> (),
    >(
        &self,
        mut idle_strategy_func: AeronIdleStrategyFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_idle_strategy(
                self.get_inner(),
                Some(
                    aeron_idle_strategy_func_t_callback_for_once_closure::<
                        AeronIdleStrategyFuncHandlerImpl,
                    >,
                ),
                &mut idle_strategy_func as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the various credentials callbacks to use when connecting to the Aeron Archive."]
    pub fn set_credentials_supplier<
        AeronArchiveCredentialsFreeFuncHandlerImpl: AeronArchiveCredentialsFreeFuncCallback,
    >(
        &self,
        encoded_credentials: aeron_archive_credentials_encoded_credentials_supplier_func_t,
        on_challenge: aeron_archive_credentials_challenge_supplier_func_t,
        on_free: Option<&Handler<AeronArchiveCredentialsFreeFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_credentials_supplier(
                self.get_inner(),
                encoded_credentials.into(),
                on_challenge.into(),
                {
                    let callback: aeron_archive_credentials_free_func_t = if on_free.is_none() {
                        None
                    } else {
                        Some(
                            aeron_archive_credentials_free_func_t_callback::<
                                AeronArchiveCredentialsFreeFuncHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                on_free
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the various credentials callbacks to use when connecting to the Aeron Archive."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_credentials_supplier_once<
        AeronArchiveCredentialsFreeFuncHandlerImpl: FnMut(AeronArchiveEncodedCredentials) -> (),
    >(
        &self,
        encoded_credentials: aeron_archive_credentials_encoded_credentials_supplier_func_t,
        on_challenge: aeron_archive_credentials_challenge_supplier_func_t,
        mut on_free: AeronArchiveCredentialsFreeFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_credentials_supplier(
                self.get_inner(),
                encoded_credentials.into(),
                on_challenge.into(),
                Some(
                    aeron_archive_credentials_free_func_t_callback_for_once_closure::<
                        AeronArchiveCredentialsFreeFuncHandlerImpl,
                    >,
                ),
                &mut on_free as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to which recording signals are dispatched while polling for control responses."]
    pub fn set_recording_signal_consumer<
        AeronArchiveRecordingSignalConsumerFuncHandlerImpl: AeronArchiveRecordingSignalConsumerFuncCallback,
    >(
        &self,
        on_recording_signal: Option<&Handler<AeronArchiveRecordingSignalConsumerFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_recording_signal_consumer(
                self.get_inner(),
                {
                    let callback: aeron_archive_recording_signal_consumer_func_t =
                        if on_recording_signal.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_recording_signal_consumer_func_t_callback::<
                                    AeronArchiveRecordingSignalConsumerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                on_recording_signal
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to which recording signals are dispatched while polling for control responses."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_recording_signal_consumer_once<
        AeronArchiveRecordingSignalConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingSignal) -> (),
    >(
        &self,
        mut on_recording_signal: AeronArchiveRecordingSignalConsumerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_recording_signal_consumer(
                self.get_inner(),
                Some(
                    aeron_archive_recording_signal_consumer_func_t_callback_for_once_closure::<
                        AeronArchiveRecordingSignalConsumerFuncHandlerImpl,
                    >,
                ),
                &mut on_recording_signal as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to which errors are dispatched while executing archive client commands."]
    pub fn set_error_handler<AeronErrorHandlerHandlerImpl: AeronErrorHandlerCallback>(
        &self,
        error_handler: Option<&Handler<AeronErrorHandlerHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_error_handler(
                self.get_inner(),
                {
                    let callback: aeron_error_handler_t = if error_handler.is_none() {
                        None
                    } else {
                        Some(aeron_error_handler_t_callback::<AeronErrorHandlerHandlerImpl>)
                    };
                    callback
                },
                error_handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to which errors are dispatched while executing archive client commands."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_error_handler_once<
        AeronErrorHandlerHandlerImpl: FnMut(::std::os::raw::c_int, &str) -> (),
    >(
        &self,
        mut error_handler: AeronErrorHandlerHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_error_handler(
                self.get_inner(),
                Some(
                    aeron_error_handler_t_callback_for_once_closure::<AeronErrorHandlerHandlerImpl>,
                ),
                &mut error_handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to be invoked in addition to any invoker used by the Aeron instance."]
    #[doc = " \n"]
    #[doc = " Useful when running in a low thread count environment."]
    pub fn set_delegating_invoker<
        AeronArchiveDelegatingInvokerFuncHandlerImpl: AeronArchiveDelegatingInvokerFuncCallback,
    >(
        &self,
        delegating_invoker_func: Option<&Handler<AeronArchiveDelegatingInvokerFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_delegating_invoker(
                self.get_inner(),
                {
                    let callback: aeron_archive_delegating_invoker_func_t =
                        if delegating_invoker_func.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_delegating_invoker_func_t_callback::<
                                    AeronArchiveDelegatingInvokerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                delegating_invoker_func
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Specify the callback to be invoked in addition to any invoker used by the Aeron instance."]
    #[doc = " \n"]
    #[doc = " Useful when running in a low thread count environment."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_delegating_invoker_once<
        AeronArchiveDelegatingInvokerFuncHandlerImpl: FnMut() -> (),
    >(
        &self,
        mut delegating_invoker_func: AeronArchiveDelegatingInvokerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_context_set_delegating_invoker(
                self.get_inner(),
                Some(
                    aeron_archive_delegating_invoker_func_t_callback_for_once_closure::<
                        AeronArchiveDelegatingInvokerFuncHandlerImpl,
                    >,
                ),
                &mut delegating_invoker_func as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_context_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_context_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_context_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveContext {
    type Target = aeron_archive_context_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_context_t> for AeronArchiveContext {
    #[inline]
    fn from(value: *mut aeron_archive_context_t) -> Self {
        AeronArchiveContext {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveContext> for *mut aeron_archive_context_t {
    #[inline]
    fn from(value: AeronArchiveContext) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveContext> for *mut aeron_archive_context_t {
    #[inline]
    fn from(value: &AeronArchiveContext) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveContext> for aeron_archive_context_t {
    #[inline]
    fn from(value: AeronArchiveContext) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_context_t> for AeronArchiveContext {
    #[inline]
    fn from(value: *const aeron_archive_context_t) -> Self {
        AeronArchiveContext {
            inner: CResource::Borrowed(value as *mut aeron_archive_context_t),
        }
    }
}
impl From<aeron_archive_context_t> for AeronArchiveContext {
    #[inline]
    fn from(value: aeron_archive_context_t) -> Self {
        AeronArchiveContext {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronArchiveControlResponsePoller {
    inner: CResource<aeron_archive_control_response_poller_t>,
}
impl core::fmt::Debug for AeronArchiveControlResponsePoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveControlResponsePoller))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveControlResponsePoller))
                .field("inner", &self.inner)
                .field(stringify!(error_on_fragment), &self.error_on_fragment())
                .field(stringify!(control_session_id), &self.control_session_id())
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(stringify!(relevant_id), &self.relevant_id())
                .field(stringify!(recording_id), &self.recording_id())
                .field(stringify!(subscription_id), &self.subscription_id())
                .field(stringify!(position), &self.position())
                .field(
                    stringify!(recording_signal_code),
                    &self.recording_signal_code(),
                )
                .field(stringify!(version), &self.version())
                .field(
                    stringify!(error_message_malloced_len),
                    &self.error_message_malloced_len(),
                )
                .field(
                    stringify!(encoded_challenge_buffer_malloced_len),
                    &self.encoded_challenge_buffer_malloced_len(),
                )
                .field(stringify!(encoded_challenge), &self.encoded_challenge())
                .field(stringify!(is_poll_complete), &self.is_poll_complete())
                .field(stringify!(is_code_ok), &self.is_code_ok())
                .field(stringify!(is_code_error), &self.is_code_error())
                .field(stringify!(is_control_response), &self.is_control_response())
                .field(stringify!(was_challenged), &self.was_challenged())
                .field(stringify!(is_recording_signal), &self.is_recording_signal())
                .finish()
        }
    }
}
impl AeronArchiveControlResponsePoller {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_control_response_poller_t)
                );
                let inst: aeron_archive_control_response_poller_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_control_response_poller_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_control_response_poller_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn subscription(&self) -> AeronSubscription {
        self.subscription.into()
    }
    #[inline]
    pub fn fragment_limit(&self) -> ::std::os::raw::c_int {
        self.fragment_limit.into()
    }
    #[inline]
    pub fn fragment_assembler(&self) -> AeronControlledFragmentAssembler {
        self.fragment_assembler.into()
    }
    #[inline]
    pub fn error_on_fragment(&self) -> bool {
        self.error_on_fragment.into()
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn relevant_id(&self) -> i64 {
        self.relevant_id.into()
    }
    #[inline]
    pub fn recording_id(&self) -> i64 {
        self.recording_id.into()
    }
    #[inline]
    pub fn subscription_id(&self) -> i64 {
        self.subscription_id.into()
    }
    #[inline]
    pub fn position(&self) -> i64 {
        self.position.into()
    }
    #[inline]
    pub fn recording_signal_code(&self) -> i32 {
        self.recording_signal_code.into()
    }
    #[inline]
    pub fn version(&self) -> i32 {
        self.version.into()
    }
    #[inline]
    pub fn error_message(&self) -> &str {
        if self.error_message.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.error_message)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn error_message_malloced_len(&self) -> u32 {
        self.error_message_malloced_len.into()
    }
    #[inline]
    pub fn encoded_challenge_buffer(&self) -> &str {
        if self.encoded_challenge_buffer.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.encoded_challenge_buffer)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn encoded_challenge_buffer_malloced_len(&self) -> u32 {
        self.encoded_challenge_buffer_malloced_len.into()
    }
    #[inline]
    pub fn encoded_challenge(&self) -> AeronArchiveEncodedCredentials {
        self.encoded_challenge.into()
    }
    #[inline]
    pub fn code_value(&self) -> ::std::os::raw::c_int {
        self.code_value.into()
    }
    #[inline]
    pub fn is_poll_complete(&self) -> bool {
        self.is_poll_complete.into()
    }
    #[inline]
    pub fn is_code_ok(&self) -> bool {
        self.is_code_ok.into()
    }
    #[inline]
    pub fn is_code_error(&self) -> bool {
        self.is_code_error.into()
    }
    #[inline]
    pub fn is_control_response(&self) -> bool {
        self.is_control_response.into()
    }
    #[inline]
    pub fn was_challenged(&self) -> bool {
        self.was_challenged.into()
    }
    #[inline]
    pub fn is_recording_signal(&self) -> bool {
        self.is_recording_signal.into()
    }
    #[inline]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_control_response_poller_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_control_response_poller_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_control_response_poller_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_control_response_poller_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_control_response_poller_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveControlResponsePoller {
    type Target = aeron_archive_control_response_poller_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_control_response_poller_t> for AeronArchiveControlResponsePoller {
    #[inline]
    fn from(value: *mut aeron_archive_control_response_poller_t) -> Self {
        AeronArchiveControlResponsePoller {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveControlResponsePoller> for *mut aeron_archive_control_response_poller_t {
    #[inline]
    fn from(value: AeronArchiveControlResponsePoller) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveControlResponsePoller> for *mut aeron_archive_control_response_poller_t {
    #[inline]
    fn from(value: &AeronArchiveControlResponsePoller) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveControlResponsePoller> for aeron_archive_control_response_poller_t {
    #[inline]
    fn from(value: AeronArchiveControlResponsePoller) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_control_response_poller_t> for AeronArchiveControlResponsePoller {
    #[inline]
    fn from(value: *const aeron_archive_control_response_poller_t) -> Self {
        AeronArchiveControlResponsePoller {
            inner: CResource::Borrowed(value as *mut aeron_archive_control_response_poller_t),
        }
    }
}
impl From<aeron_archive_control_response_poller_t> for AeronArchiveControlResponsePoller {
    #[inline]
    fn from(value: aeron_archive_control_response_poller_t) -> Self {
        AeronArchiveControlResponsePoller {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronArchiveControlResponsePoller {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!(
                        "auto closing {}",
                        stringify!(AeronArchiveControlResponsePoller)
                    );
                    let result = self.close();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!(
                        "{} not closed",
                        stringify!(AeronArchiveControlResponsePoller)
                    );
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct AeronArchiveEncodedCredentials {
    inner: CResource<aeron_archive_encoded_credentials_t>,
}
impl core::fmt::Debug for AeronArchiveEncodedCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveEncodedCredentials))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveEncodedCredentials))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronArchiveEncodedCredentials {
    #[inline]
    pub fn new(data: &std::ffi::CStr, length: u32) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_encoded_credentials_t {
                    data: data.as_ptr(),
                    length: length.into(),
                };
                let inner_ptr: *mut aeron_archive_encoded_credentials_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_encoded_credentials_t)
                );
                let inst: aeron_archive_encoded_credentials_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_encoded_credentials_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_encoded_credentials_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn data(&self) -> &str {
        if self.data.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.data).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn length(&self) -> u32 {
        self.length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_encoded_credentials_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_encoded_credentials_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_encoded_credentials_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveEncodedCredentials {
    type Target = aeron_archive_encoded_credentials_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_encoded_credentials_t> for AeronArchiveEncodedCredentials {
    #[inline]
    fn from(value: *mut aeron_archive_encoded_credentials_t) -> Self {
        AeronArchiveEncodedCredentials {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveEncodedCredentials> for *mut aeron_archive_encoded_credentials_t {
    #[inline]
    fn from(value: AeronArchiveEncodedCredentials) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveEncodedCredentials> for *mut aeron_archive_encoded_credentials_t {
    #[inline]
    fn from(value: &AeronArchiveEncodedCredentials) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveEncodedCredentials> for aeron_archive_encoded_credentials_t {
    #[inline]
    fn from(value: AeronArchiveEncodedCredentials) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_encoded_credentials_t> for AeronArchiveEncodedCredentials {
    #[inline]
    fn from(value: *const aeron_archive_encoded_credentials_t) -> Self {
        AeronArchiveEncodedCredentials {
            inner: CResource::Borrowed(value as *mut aeron_archive_encoded_credentials_t),
        }
    }
}
impl From<aeron_archive_encoded_credentials_t> for AeronArchiveEncodedCredentials {
    #[inline]
    fn from(value: aeron_archive_encoded_credentials_t) -> Self {
        AeronArchiveEncodedCredentials {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveEncodedCredentials {
    fn default() -> Self {
        AeronArchiveEncodedCredentials::new_zeroed_on_heap()
    }
}
impl AeronArchiveEncodedCredentials {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronArchiveProxy {
    inner: CResource<aeron_archive_proxy_t>,
    _ctx: Option<AeronArchiveContext>,
    _exclusive_publication: Option<AeronExclusivePublication>,
}
impl core::fmt::Debug for AeronArchiveProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveProxy))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveProxy))
                .field("inner", &self.inner)
                .field(stringify!(control_session_id), &self.control_session_id())
                .finish()
        }
    }
}
impl AeronArchiveProxy {
    pub fn new(
        ctx: &AeronArchiveContext,
        exclusive_publication: &AeronExclusivePublication,
        retry_attempts: ::std::os::raw::c_int,
    ) -> Result<Self, AeronCError> {
        let ctx_copy = ctx.clone();
        let ctx: *mut aeron_archive_context_t = ctx.into();
        let exclusive_publication_copy = exclusive_publication.clone();
        let exclusive_publication: *mut aeron_exclusive_publication_t =
            exclusive_publication.into();
        let retry_attempts: ::std::os::raw::c_int = retry_attempts.into();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_archive_proxy_create(ctx_field, ctx, exclusive_publication, retry_attempts)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_archive_proxy_delete(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
            _ctx: Some(ctx_copy),
            _exclusive_publication: Some(exclusive_publication_copy),
        })
    }
    #[inline]
    pub fn ctx(&self) -> AeronArchiveContext {
        self.ctx.into()
    }
    #[inline]
    pub fn exclusive_publication(&self) -> AeronExclusivePublication {
        self.exclusive_publication.into()
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn retry_attempts(&self) -> ::std::os::raw::c_int {
        self.retry_attempts.into()
    }
    #[inline]
    pub fn buffer(&self) -> [u8; 8192usize] {
        self.buffer.into()
    }
    #[inline]
    pub fn init(
        &self,
        ctx: &AeronArchiveContext,
        exclusive_publication: &AeronExclusivePublication,
        retry_attempts: ::std::os::raw::c_int,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_proxy_init(
                self.get_inner(),
                ctx.get_inner(),
                exclusive_publication.get_inner(),
                retry_attempts.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn set_control_esssion_id(&self, control_session_id: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_proxy_set_control_esssion_id(
                self.get_inner(),
                control_session_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_proxy_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn delete(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_proxy_delete(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn try_connect(
        &self,
        control_response_channel: &std::ffi::CStr,
        control_response_stream_id: i32,
        encoded_credentials: &AeronArchiveEncodedCredentials,
        correlation_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_try_connect(
                self.get_inner(),
                control_response_channel.as_ptr(),
                control_response_stream_id.into(),
                encoded_credentials.get_inner(),
                correlation_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn archive_id(&self, correlation_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_archive_id(self.get_inner(), correlation_id.into());
            result.into()
        }
    }
    #[inline]
    pub fn challenge_response(
        &self,
        encoded_credentials: &AeronArchiveEncodedCredentials,
        correlation_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_challenge_response(
                self.get_inner(),
                encoded_credentials.get_inner(),
                correlation_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn close_session(&self) -> bool {
        unsafe {
            let result = aeron_archive_proxy_close_session(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn start_recording(
        &self,
        recording_channel: &std::ffi::CStr,
        recording_stream_id: i32,
        local_source: bool,
        auto_stop: bool,
        correlation_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_start_recording(
                self.get_inner(),
                recording_channel.as_ptr(),
                recording_stream_id.into(),
                local_source.into(),
                auto_stop.into(),
                correlation_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn get_recording_position(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_get_recording_position(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn get_start_position(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_get_start_position(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn get_stop_position(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_get_stop_position(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn get_max_recorded_position(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_get_max_recorded_position(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_recording(
        &self,
        correlation_id: i64,
        channel: &std::ffi::CStr,
        stream_id: i32,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_recording(
                self.get_inner(),
                correlation_id.into(),
                channel.as_ptr(),
                stream_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_recording_subscription(&self, correlation_id: i64, subscription_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_recording_subscription(
                self.get_inner(),
                correlation_id.into(),
                subscription_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_recording_by_identity(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_recording_by_identity(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn find_last_matching_recording(
        &self,
        correlation_id: i64,
        min_recording_id: i64,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        session_id: i32,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_find_last_matching_recording(
                self.get_inner(),
                correlation_id.into(),
                min_recording_id.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                session_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn list_recording(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_list_recording(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn list_recordings(
        &self,
        correlation_id: i64,
        from_recording_id: i64,
        record_count: i32,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_list_recordings(
                self.get_inner(),
                correlation_id.into(),
                from_recording_id.into(),
                record_count.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn list_recordings_for_uri(
        &self,
        correlation_id: i64,
        from_recording_id: i64,
        record_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_list_recordings_for_uri(
                self.get_inner(),
                correlation_id.into(),
                from_recording_id.into(),
                record_count.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn replay(
        &self,
        correlation_id: i64,
        recording_id: i64,
        replay_channel: &std::ffi::CStr,
        replay_stream_id: i32,
        params: &AeronArchiveReplayParams,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_replay(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
                replay_channel.as_ptr(),
                replay_stream_id.into(),
                params.get_inner(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn truncate_recording(
        &self,
        correlation_id: i64,
        recording_id: i64,
        position: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_truncate_recording(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
                position.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_replay(&self, correlation_id: i64, replay_session_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_replay(
                self.get_inner(),
                correlation_id.into(),
                replay_session_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_all_replays(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_all_replays(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn list_recording_subscriptions(
        &self,
        correlation_id: i64,
        pseudo_index: i32,
        subscription_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        apply_stream_id: bool,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_list_recording_subscriptions(
                self.get_inner(),
                correlation_id.into(),
                pseudo_index.into(),
                subscription_count.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                apply_stream_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn purge_recording(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_purge_recording(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn extend_recording(
        &self,
        recording_id: i64,
        recording_channel: &std::ffi::CStr,
        recording_stream_id: i32,
        local_source: bool,
        auto_stop: bool,
        correlation_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_extend_recording(
                self.get_inner(),
                recording_id.into(),
                recording_channel.as_ptr(),
                recording_stream_id.into(),
                local_source.into(),
                auto_stop.into(),
                correlation_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn replicate(
        &self,
        correlation_id: i64,
        src_recording_id: i64,
        src_control_stream_id: i32,
        src_control_channel: &std::ffi::CStr,
        params: &AeronArchiveReplicationParams,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_replicate(
                self.get_inner(),
                correlation_id.into(),
                src_recording_id.into(),
                src_control_stream_id.into(),
                src_control_channel.as_ptr(),
                params.get_inner(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn stop_replication(&self, correlation_id: i64, replication_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_stop_replication(
                self.get_inner(),
                correlation_id.into(),
                replication_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn aeron_archive_request_replay_token(
        &self,
        correlation_id: i64,
        recording_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_request_replay_token(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn detach_segments(
        &self,
        correlation_id: i64,
        recording_id: i64,
        new_start_position: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_detach_segments(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
                new_start_position.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn delete_detached_segments(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_delete_detached_segments(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn purge_segments(
        &self,
        correlation_id: i64,
        recording_id: i64,
        new_start_position: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_purge_segments(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
                new_start_position.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn attach_segments(&self, correlation_id: i64, recording_id: i64) -> bool {
        unsafe {
            let result = aeron_archive_proxy_attach_segments(
                self.get_inner(),
                correlation_id.into(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn migrate_segments(
        &self,
        correlation_id: i64,
        src_recording_id: i64,
        dst_recording_id: i64,
    ) -> bool {
        unsafe {
            let result = aeron_archive_proxy_migrate_segments(
                self.get_inner(),
                correlation_id.into(),
                src_recording_id.into(),
                dst_recording_id.into(),
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_proxy_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_proxy_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_proxy_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveProxy {
    type Target = aeron_archive_proxy_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_proxy_t> for AeronArchiveProxy {
    #[inline]
    fn from(value: *mut aeron_archive_proxy_t) -> Self {
        AeronArchiveProxy {
            inner: CResource::Borrowed(value),
            _ctx: None,
            _exclusive_publication: None,
        }
    }
}
impl From<AeronArchiveProxy> for *mut aeron_archive_proxy_t {
    #[inline]
    fn from(value: AeronArchiveProxy) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveProxy> for *mut aeron_archive_proxy_t {
    #[inline]
    fn from(value: &AeronArchiveProxy) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveProxy> for aeron_archive_proxy_t {
    #[inline]
    fn from(value: AeronArchiveProxy) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_proxy_t> for AeronArchiveProxy {
    #[inline]
    fn from(value: *const aeron_archive_proxy_t) -> Self {
        AeronArchiveProxy {
            inner: CResource::Borrowed(value as *mut aeron_archive_proxy_t),
            _ctx: None,
            _exclusive_publication: None,
        }
    }
}
impl From<aeron_archive_proxy_t> for AeronArchiveProxy {
    #[inline]
    fn from(value: aeron_archive_proxy_t) -> Self {
        AeronArchiveProxy {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
            _ctx: None,
            _exclusive_publication: None,
        }
    }
}
#[derive(Clone)]
pub struct AeronArchiveRecordingDescriptorPoller {
    inner: CResource<aeron_archive_recording_descriptor_poller_t>,
}
impl core::fmt::Debug for AeronArchiveRecordingDescriptorPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveRecordingDescriptorPoller))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveRecordingDescriptorPoller))
                .field("inner", &self.inner)
                .field(stringify!(control_session_id), &self.control_session_id())
                .field(stringify!(error_on_fragment), &self.error_on_fragment())
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(
                    stringify!(remaining_record_count),
                    &self.remaining_record_count(),
                )
                .field(
                    stringify!(is_dispatch_complete),
                    &self.is_dispatch_complete(),
                )
                .finish()
        }
    }
}
impl AeronArchiveRecordingDescriptorPoller {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_recording_descriptor_poller_t)
                );
                let inst: aeron_archive_recording_descriptor_poller_t =
                    unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_recording_descriptor_poller_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_recording_descriptor_poller_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn ctx(&self) -> AeronArchiveContext {
        self.ctx.into()
    }
    #[inline]
    pub fn subscription(&self) -> AeronSubscription {
        self.subscription.into()
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn fragment_limit(&self) -> ::std::os::raw::c_int {
        self.fragment_limit.into()
    }
    #[inline]
    pub fn fragment_assembler(&self) -> AeronControlledFragmentAssembler {
        self.fragment_assembler.into()
    }
    #[inline]
    pub fn error_on_fragment(&self) -> bool {
        self.error_on_fragment.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn remaining_record_count(&self) -> i32 {
        self.remaining_record_count.into()
    }
    #[inline]
    pub fn recording_descriptor_consumer(
        &self,
    ) -> aeron_archive_recording_descriptor_consumer_func_t {
        self.recording_descriptor_consumer.into()
    }
    #[inline]
    pub fn recording_descriptor_consumer_clientd(&self) -> *mut ::std::os::raw::c_void {
        self.recording_descriptor_consumer_clientd.into()
    }
    #[inline]
    pub fn is_dispatch_complete(&self) -> bool {
        self.is_dispatch_complete.into()
    }
    #[inline]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_recording_descriptor_poller_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn reset<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingDescriptorConsumerFuncCallback,
    >(
        &self,
        correlation_id: i64,
        record_count: i32,
        recording_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> () {
        unsafe {
            let result = aeron_archive_recording_descriptor_poller_reset(
                self.get_inner(),
                correlation_id.into(),
                record_count.into(),
                {
                    let callback: aeron_archive_recording_descriptor_consumer_func_t =
                        if recording_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_recording_descriptor_consumer_func_t_callback::<
                                    AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                recording_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn reset_once<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingDescriptor) -> (),
    >(
        &self,
        correlation_id: i64,
        record_count: i32,
        mut recording_descriptor_consumer: AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
    ) -> () {
        unsafe {
            let result = aeron_archive_recording_descriptor_poller_reset(
                self.get_inner(),
                correlation_id.into(),
                record_count.into(),
                Some(
                    aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure::<
                        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                    >,
                ),
                &mut recording_descriptor_consumer as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    pub fn poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_recording_descriptor_poller_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_recording_descriptor_poller_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_recording_descriptor_poller_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_recording_descriptor_poller_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveRecordingDescriptorPoller {
    type Target = aeron_archive_recording_descriptor_poller_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_recording_descriptor_poller_t>
    for AeronArchiveRecordingDescriptorPoller
{
    #[inline]
    fn from(value: *mut aeron_archive_recording_descriptor_poller_t) -> Self {
        AeronArchiveRecordingDescriptorPoller {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveRecordingDescriptorPoller>
    for *mut aeron_archive_recording_descriptor_poller_t
{
    #[inline]
    fn from(value: AeronArchiveRecordingDescriptorPoller) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveRecordingDescriptorPoller>
    for *mut aeron_archive_recording_descriptor_poller_t
{
    #[inline]
    fn from(value: &AeronArchiveRecordingDescriptorPoller) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveRecordingDescriptorPoller> for aeron_archive_recording_descriptor_poller_t {
    #[inline]
    fn from(value: AeronArchiveRecordingDescriptorPoller) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_recording_descriptor_poller_t>
    for AeronArchiveRecordingDescriptorPoller
{
    #[inline]
    fn from(value: *const aeron_archive_recording_descriptor_poller_t) -> Self {
        AeronArchiveRecordingDescriptorPoller {
            inner: CResource::Borrowed(value as *mut aeron_archive_recording_descriptor_poller_t),
        }
    }
}
impl From<aeron_archive_recording_descriptor_poller_t> for AeronArchiveRecordingDescriptorPoller {
    #[inline]
    fn from(value: aeron_archive_recording_descriptor_poller_t) -> Self {
        AeronArchiveRecordingDescriptorPoller {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronArchiveRecordingDescriptorPoller {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!(
                        "auto closing {}",
                        stringify!(AeronArchiveRecordingDescriptorPoller)
                    );
                    let result = self.close();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!(
                        "{} not closed",
                        stringify!(AeronArchiveRecordingDescriptorPoller)
                    );
                }
            }
        }
    }
}
#[doc = "Struct containing the details of a recording"]
#[derive(Clone)]
pub struct AeronArchiveRecordingDescriptor {
    inner: CResource<aeron_archive_recording_descriptor_t>,
}
impl core::fmt::Debug for AeronArchiveRecordingDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveRecordingDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveRecordingDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(control_session_id), &self.control_session_id())
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(stringify!(recording_id), &self.recording_id())
                .field(stringify!(start_timestamp), &self.start_timestamp())
                .field(stringify!(stop_timestamp), &self.stop_timestamp())
                .field(stringify!(start_position), &self.start_position())
                .field(stringify!(stop_position), &self.stop_position())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(stringify!(segment_file_length), &self.segment_file_length())
                .field(stringify!(term_buffer_length), &self.term_buffer_length())
                .field(stringify!(mtu_length), &self.mtu_length())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(
                    stringify!(stripped_channel_length),
                    &self.stripped_channel_length(),
                )
                .field(
                    stringify!(original_channel_length),
                    &self.original_channel_length(),
                )
                .field(
                    stringify!(source_identity_length),
                    &self.source_identity_length(),
                )
                .finish()
        }
    }
}
impl AeronArchiveRecordingDescriptor {
    #[inline]
    pub fn new(
        control_session_id: i64,
        correlation_id: i64,
        recording_id: i64,
        start_timestamp: i64,
        stop_timestamp: i64,
        start_position: i64,
        stop_position: i64,
        initial_term_id: i32,
        segment_file_length: i32,
        term_buffer_length: i32,
        mtu_length: i32,
        session_id: i32,
        stream_id: i32,
        stripped_channel: *mut ::std::os::raw::c_char,
        stripped_channel_length: usize,
        original_channel: *mut ::std::os::raw::c_char,
        original_channel_length: usize,
        source_identity: *mut ::std::os::raw::c_char,
        source_identity_length: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_recording_descriptor_t {
                    control_session_id: control_session_id.into(),
                    correlation_id: correlation_id.into(),
                    recording_id: recording_id.into(),
                    start_timestamp: start_timestamp.into(),
                    stop_timestamp: stop_timestamp.into(),
                    start_position: start_position.into(),
                    stop_position: stop_position.into(),
                    initial_term_id: initial_term_id.into(),
                    segment_file_length: segment_file_length.into(),
                    term_buffer_length: term_buffer_length.into(),
                    mtu_length: mtu_length.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    stripped_channel: stripped_channel.into(),
                    stripped_channel_length: stripped_channel_length.into(),
                    original_channel: original_channel.into(),
                    original_channel_length: original_channel_length.into(),
                    source_identity: source_identity.into(),
                    source_identity_length: source_identity_length.into(),
                };
                let inner_ptr: *mut aeron_archive_recording_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_recording_descriptor_t)
                );
                let inst: aeron_archive_recording_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_recording_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_recording_descriptor_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn recording_id(&self) -> i64 {
        self.recording_id.into()
    }
    #[inline]
    pub fn start_timestamp(&self) -> i64 {
        self.start_timestamp.into()
    }
    #[inline]
    pub fn stop_timestamp(&self) -> i64 {
        self.stop_timestamp.into()
    }
    #[inline]
    pub fn start_position(&self) -> i64 {
        self.start_position.into()
    }
    #[inline]
    pub fn stop_position(&self) -> i64 {
        self.stop_position.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn segment_file_length(&self) -> i32 {
        self.segment_file_length.into()
    }
    #[inline]
    pub fn term_buffer_length(&self) -> i32 {
        self.term_buffer_length.into()
    }
    #[inline]
    pub fn mtu_length(&self) -> i32 {
        self.mtu_length.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn stripped_channel(&self) -> &str {
        if self.stripped_channel.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.stripped_channel)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn stripped_channel_length(&self) -> usize {
        self.stripped_channel_length.into()
    }
    #[inline]
    pub fn original_channel(&self) -> &str {
        if self.original_channel.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.original_channel)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn original_channel_length(&self) -> usize {
        self.original_channel_length.into()
    }
    #[inline]
    pub fn source_identity(&self) -> &str {
        if self.source_identity.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.source_identity)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn source_identity_length(&self) -> usize {
        self.source_identity_length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_recording_descriptor_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_recording_descriptor_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_recording_descriptor_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveRecordingDescriptor {
    type Target = aeron_archive_recording_descriptor_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_recording_descriptor_t> for AeronArchiveRecordingDescriptor {
    #[inline]
    fn from(value: *mut aeron_archive_recording_descriptor_t) -> Self {
        AeronArchiveRecordingDescriptor {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveRecordingDescriptor> for *mut aeron_archive_recording_descriptor_t {
    #[inline]
    fn from(value: AeronArchiveRecordingDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveRecordingDescriptor> for *mut aeron_archive_recording_descriptor_t {
    #[inline]
    fn from(value: &AeronArchiveRecordingDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveRecordingDescriptor> for aeron_archive_recording_descriptor_t {
    #[inline]
    fn from(value: AeronArchiveRecordingDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_recording_descriptor_t> for AeronArchiveRecordingDescriptor {
    #[inline]
    fn from(value: *const aeron_archive_recording_descriptor_t) -> Self {
        AeronArchiveRecordingDescriptor {
            inner: CResource::Borrowed(value as *mut aeron_archive_recording_descriptor_t),
        }
    }
}
impl From<aeron_archive_recording_descriptor_t> for AeronArchiveRecordingDescriptor {
    #[inline]
    fn from(value: aeron_archive_recording_descriptor_t) -> Self {
        AeronArchiveRecordingDescriptor {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveRecordingDescriptor {
    fn default() -> Self {
        AeronArchiveRecordingDescriptor::new_zeroed_on_heap()
    }
}
impl AeronArchiveRecordingDescriptor {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[doc = "Struct containing the details of a recording signal."]
#[derive(Clone)]
pub struct AeronArchiveRecordingSignal {
    inner: CResource<aeron_archive_recording_signal_t>,
}
impl core::fmt::Debug for AeronArchiveRecordingSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveRecordingSignal))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveRecordingSignal))
                .field("inner", &self.inner)
                .field(stringify!(control_session_id), &self.control_session_id())
                .field(stringify!(recording_id), &self.recording_id())
                .field(stringify!(subscription_id), &self.subscription_id())
                .field(stringify!(position), &self.position())
                .field(
                    stringify!(recording_signal_code),
                    &self.recording_signal_code(),
                )
                .finish()
        }
    }
}
impl AeronArchiveRecordingSignal {
    #[inline]
    pub fn new(
        control_session_id: i64,
        recording_id: i64,
        subscription_id: i64,
        position: i64,
        recording_signal_code: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_recording_signal_t {
                    control_session_id: control_session_id.into(),
                    recording_id: recording_id.into(),
                    subscription_id: subscription_id.into(),
                    position: position.into(),
                    recording_signal_code: recording_signal_code.into(),
                };
                let inner_ptr: *mut aeron_archive_recording_signal_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_recording_signal_t)
                );
                let inst: aeron_archive_recording_signal_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_recording_signal_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_recording_signal_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn recording_id(&self) -> i64 {
        self.recording_id.into()
    }
    #[inline]
    pub fn subscription_id(&self) -> i64 {
        self.subscription_id.into()
    }
    #[inline]
    pub fn position(&self) -> i64 {
        self.position.into()
    }
    #[inline]
    pub fn recording_signal_code(&self) -> i32 {
        self.recording_signal_code.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_recording_signal_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_recording_signal_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_recording_signal_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveRecordingSignal {
    type Target = aeron_archive_recording_signal_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_recording_signal_t> for AeronArchiveRecordingSignal {
    #[inline]
    fn from(value: *mut aeron_archive_recording_signal_t) -> Self {
        AeronArchiveRecordingSignal {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveRecordingSignal> for *mut aeron_archive_recording_signal_t {
    #[inline]
    fn from(value: AeronArchiveRecordingSignal) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveRecordingSignal> for *mut aeron_archive_recording_signal_t {
    #[inline]
    fn from(value: &AeronArchiveRecordingSignal) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveRecordingSignal> for aeron_archive_recording_signal_t {
    #[inline]
    fn from(value: AeronArchiveRecordingSignal) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_recording_signal_t> for AeronArchiveRecordingSignal {
    #[inline]
    fn from(value: *const aeron_archive_recording_signal_t) -> Self {
        AeronArchiveRecordingSignal {
            inner: CResource::Borrowed(value as *mut aeron_archive_recording_signal_t),
        }
    }
}
impl From<aeron_archive_recording_signal_t> for AeronArchiveRecordingSignal {
    #[inline]
    fn from(value: aeron_archive_recording_signal_t) -> Self {
        AeronArchiveRecordingSignal {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveRecordingSignal {
    fn default() -> Self {
        AeronArchiveRecordingSignal::new_zeroed_on_heap()
    }
}
impl AeronArchiveRecordingSignal {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronArchiveRecordingSubscriptionDescriptorPoller {
    inner: CResource<aeron_archive_recording_subscription_descriptor_poller_t>,
}
impl core::fmt::Debug for AeronArchiveRecordingSubscriptionDescriptorPoller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(
                AeronArchiveRecordingSubscriptionDescriptorPoller
            ))
            .field("inner", &"null")
            .finish()
        } else {
            f.debug_struct(stringify!(
                AeronArchiveRecordingSubscriptionDescriptorPoller
            ))
            .field("inner", &self.inner)
            .field(stringify!(control_session_id), &self.control_session_id())
            .field(stringify!(error_on_fragment), &self.error_on_fragment())
            .field(stringify!(correlation_id), &self.correlation_id())
            .field(
                stringify!(remaining_subscription_count),
                &self.remaining_subscription_count(),
            )
            .field(
                stringify!(is_dispatch_complete),
                &self.is_dispatch_complete(),
            )
            .finish()
        }
    }
}
impl AeronArchiveRecordingSubscriptionDescriptorPoller {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_recording_subscription_descriptor_poller_t)
                );
                let inst: aeron_archive_recording_subscription_descriptor_poller_t =
                    unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_recording_subscription_descriptor_poller_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_recording_subscription_descriptor_poller_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn ctx(&self) -> AeronArchiveContext {
        self.ctx.into()
    }
    #[inline]
    pub fn subscription(&self) -> AeronSubscription {
        self.subscription.into()
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn fragment_limit(&self) -> ::std::os::raw::c_int {
        self.fragment_limit.into()
    }
    #[inline]
    pub fn fragment_assembler(&self) -> AeronControlledFragmentAssembler {
        self.fragment_assembler.into()
    }
    #[inline]
    pub fn error_on_fragment(&self) -> bool {
        self.error_on_fragment.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn remaining_subscription_count(&self) -> i32 {
        self.remaining_subscription_count.into()
    }
    #[inline]
    pub fn recording_subscription_descriptor_consumer(
        &self,
    ) -> aeron_archive_recording_subscription_descriptor_consumer_func_t {
        self.recording_subscription_descriptor_consumer.into()
    }
    #[inline]
    pub fn recording_subscription_descriptor_consumer_clientd(
        &self,
    ) -> *mut ::std::os::raw::c_void {
        self.recording_subscription_descriptor_consumer_clientd
            .into()
    }
    #[inline]
    pub fn is_dispatch_complete(&self) -> bool {
        self.is_dispatch_complete.into()
    }
    #[inline]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result =
                aeron_archive_recording_subscription_descriptor_poller_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn reset<
        AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingSubscriptionDescriptorConsumerFuncCallback,
    >(
        &self,
        correlation_id: i64,
        subscription_count: i32,
        recording_subscription_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> () {
        unsafe {
            let result = aeron_archive_recording_subscription_descriptor_poller_reset(
                self.get_inner(),
                correlation_id.into(),
                subscription_count.into(),
                {
                    let callback: aeron_archive_recording_subscription_descriptor_consumer_func_t =
                        if recording_subscription_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some (aeron_archive_recording_subscription_descriptor_consumer_func_t_callback :: < AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl >)
                        };
                    callback
                },
                recording_subscription_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn reset_once<
        AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingSubscriptionDescriptor) -> (),
    >(
        &self,
        correlation_id: i64,
        subscription_count: i32,
        mut recording_subscription_descriptor_consumer : AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl,
    ) -> () {
        unsafe {
            let result = aeron_archive_recording_subscription_descriptor_poller_reset (self . get_inner () , correlation_id . into () , subscription_count . into () , Some (aeron_archive_recording_subscription_descriptor_consumer_func_t_callback_for_once_closure :: < AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl >) , & mut recording_subscription_descriptor_consumer as * mut _ as * mut std :: os :: raw :: c_void) ;
            result.into()
        }
    }
    #[inline]
    pub fn poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_archive_recording_subscription_descriptor_poller_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_recording_subscription_descriptor_poller_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_recording_subscription_descriptor_poller_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_recording_subscription_descriptor_poller_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveRecordingSubscriptionDescriptorPoller {
    type Target = aeron_archive_recording_subscription_descriptor_poller_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_recording_subscription_descriptor_poller_t>
    for AeronArchiveRecordingSubscriptionDescriptorPoller
{
    #[inline]
    fn from(value: *mut aeron_archive_recording_subscription_descriptor_poller_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptorPoller {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveRecordingSubscriptionDescriptorPoller>
    for *mut aeron_archive_recording_subscription_descriptor_poller_t
{
    #[inline]
    fn from(value: AeronArchiveRecordingSubscriptionDescriptorPoller) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveRecordingSubscriptionDescriptorPoller>
    for *mut aeron_archive_recording_subscription_descriptor_poller_t
{
    #[inline]
    fn from(value: &AeronArchiveRecordingSubscriptionDescriptorPoller) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveRecordingSubscriptionDescriptorPoller>
    for aeron_archive_recording_subscription_descriptor_poller_t
{
    #[inline]
    fn from(value: AeronArchiveRecordingSubscriptionDescriptorPoller) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_recording_subscription_descriptor_poller_t>
    for AeronArchiveRecordingSubscriptionDescriptorPoller
{
    #[inline]
    fn from(value: *const aeron_archive_recording_subscription_descriptor_poller_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptorPoller {
            inner: CResource::Borrowed(
                value as *mut aeron_archive_recording_subscription_descriptor_poller_t,
            ),
        }
    }
}
impl From<aeron_archive_recording_subscription_descriptor_poller_t>
    for AeronArchiveRecordingSubscriptionDescriptorPoller
{
    #[inline]
    fn from(value: aeron_archive_recording_subscription_descriptor_poller_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptorPoller {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronArchiveRecordingSubscriptionDescriptorPoller {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!(
                        "auto closing {}",
                        stringify!(AeronArchiveRecordingSubscriptionDescriptorPoller)
                    );
                    let result = self.close();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!(
                        "{} not closed",
                        stringify!(AeronArchiveRecordingSubscriptionDescriptorPoller)
                    );
                }
            }
        }
    }
}
#[doc = "Struct containing the details of a recording subscription"]
#[derive(Clone)]
pub struct AeronArchiveRecordingSubscriptionDescriptor {
    inner: CResource<aeron_archive_recording_subscription_descriptor_t>,
}
impl core::fmt::Debug for AeronArchiveRecordingSubscriptionDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveRecordingSubscriptionDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveRecordingSubscriptionDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(control_session_id), &self.control_session_id())
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(stringify!(subscription_id), &self.subscription_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(
                    stringify!(stripped_channel_length),
                    &self.stripped_channel_length(),
                )
                .finish()
        }
    }
}
impl AeronArchiveRecordingSubscriptionDescriptor {
    #[inline]
    pub fn new(
        control_session_id: i64,
        correlation_id: i64,
        subscription_id: i64,
        stream_id: i32,
        stripped_channel: *mut ::std::os::raw::c_char,
        stripped_channel_length: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_recording_subscription_descriptor_t {
                    control_session_id: control_session_id.into(),
                    correlation_id: correlation_id.into(),
                    subscription_id: subscription_id.into(),
                    stream_id: stream_id.into(),
                    stripped_channel: stripped_channel.into(),
                    stripped_channel_length: stripped_channel_length.into(),
                };
                let inner_ptr: *mut aeron_archive_recording_subscription_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_recording_subscription_descriptor_t)
                );
                let inst: aeron_archive_recording_subscription_descriptor_t =
                    unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_recording_subscription_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_recording_subscription_descriptor_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        self.control_session_id.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn subscription_id(&self) -> i64 {
        self.subscription_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn stripped_channel(&self) -> &str {
        if self.stripped_channel.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.stripped_channel)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn stripped_channel_length(&self) -> usize {
        self.stripped_channel_length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_recording_subscription_descriptor_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_recording_subscription_descriptor_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_recording_subscription_descriptor_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveRecordingSubscriptionDescriptor {
    type Target = aeron_archive_recording_subscription_descriptor_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_recording_subscription_descriptor_t>
    for AeronArchiveRecordingSubscriptionDescriptor
{
    #[inline]
    fn from(value: *mut aeron_archive_recording_subscription_descriptor_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptor {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveRecordingSubscriptionDescriptor>
    for *mut aeron_archive_recording_subscription_descriptor_t
{
    #[inline]
    fn from(value: AeronArchiveRecordingSubscriptionDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveRecordingSubscriptionDescriptor>
    for *mut aeron_archive_recording_subscription_descriptor_t
{
    #[inline]
    fn from(value: &AeronArchiveRecordingSubscriptionDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveRecordingSubscriptionDescriptor>
    for aeron_archive_recording_subscription_descriptor_t
{
    #[inline]
    fn from(value: AeronArchiveRecordingSubscriptionDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_recording_subscription_descriptor_t>
    for AeronArchiveRecordingSubscriptionDescriptor
{
    #[inline]
    fn from(value: *const aeron_archive_recording_subscription_descriptor_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptor {
            inner: CResource::Borrowed(
                value as *mut aeron_archive_recording_subscription_descriptor_t,
            ),
        }
    }
}
impl From<aeron_archive_recording_subscription_descriptor_t>
    for AeronArchiveRecordingSubscriptionDescriptor
{
    #[inline]
    fn from(value: aeron_archive_recording_subscription_descriptor_t) -> Self {
        AeronArchiveRecordingSubscriptionDescriptor {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveRecordingSubscriptionDescriptor {
    fn default() -> Self {
        AeronArchiveRecordingSubscriptionDescriptor::new_zeroed_on_heap()
    }
}
impl AeronArchiveRecordingSubscriptionDescriptor {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronArchiveReplayMerge {
    inner: CResource<aeron_archive_replay_merge_t>,
    _subscription: Option<AeronSubscription>,
    _aeron_archive: Option<AeronArchive>,
}
impl core::fmt::Debug for AeronArchiveReplayMerge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveReplayMerge))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveReplayMerge))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronArchiveReplayMerge {
    #[doc = "Create an `AeronArchiveReplayMerge` to manage the merging of a replayed stream into a live stream."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `replay_merge` the `AeronArchiveReplayMerge` to create and initialize"]
    #[doc = " \n - `subscription` the subscription to use for the replay and live stream.  Must be a multi-destination subscription"]
    #[doc = " \n - `aeron_archive` the archive client"]
    #[doc = " \n - `replay_channel` the channel to use for the replay"]
    #[doc = " \n - `replay_destination` the replay channel to use for the destination added by the subscription"]
    #[doc = " \n - `live_destination` the live stream channel to use for the destination added by the subscription"]
    #[doc = " \n - `recording_id` the recording id of the archive to replay"]
    #[doc = " \n - `start_position` the start position of the replay"]
    #[doc = " \n - `epoch_clock` the clock to use for progress checks"]
    #[doc = " \n - `merge_progress_timeout_ms` the timeout to use for progress checks"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn new(
        subscription: &AeronSubscription,
        aeron_archive: &AeronArchive,
        replay_channel: &std::ffi::CStr,
        replay_destination: &std::ffi::CStr,
        live_destination: &std::ffi::CStr,
        recording_id: i64,
        start_position: i64,
        epoch_clock: ::std::os::raw::c_longlong,
        merge_progress_timeout_ms: i64,
    ) -> Result<Self, AeronCError> {
        let subscription_copy = subscription.clone();
        let subscription: *mut aeron_subscription_t = subscription.into();
        let aeron_archive_copy = aeron_archive.clone();
        let aeron_archive: *mut aeron_archive_t = aeron_archive.into();
        let replay_channel: *const ::std::os::raw::c_char = replay_channel.as_ptr();
        let replay_destination: *const ::std::os::raw::c_char = replay_destination.as_ptr();
        let live_destination: *const ::std::os::raw::c_char = live_destination.as_ptr();
        let recording_id: i64 = recording_id.into();
        let start_position: i64 = start_position.into();
        let epoch_clock: ::std::os::raw::c_longlong = epoch_clock.into();
        let merge_progress_timeout_ms: i64 = merge_progress_timeout_ms.into();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_archive_replay_merge_init(
                    ctx_field,
                    subscription,
                    aeron_archive,
                    replay_channel,
                    replay_destination,
                    live_destination,
                    recording_id,
                    start_position,
                    epoch_clock,
                    merge_progress_timeout_ms,
                )
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_archive_replay_merge_close(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
            _subscription: Some(subscription_copy),
            _aeron_archive: Some(aeron_archive_copy),
        })
    }
    #[inline]
    #[doc = "Close and delete the `AeronArchiveReplayMerge` struct."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_replay_merge_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Process the operation of the merge.  Do not call the processing of fragments on the subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `work_count_p` an indicator of work done"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn do_work(&self, work_count_p: *mut ::std::os::raw::c_int) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_replay_merge_do_work(work_count_p.into(), self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the image used for the merging replay and live stream."]
    #[doc = " The aeron_archive_replay_merge_do_work will be called before the poll so that processing of the merge can be done."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` the handler to call for incoming fragments"]
    #[doc = " \n - `clientd` the clientd to provide to the handler"]
    #[doc = " \n - `fragment_limit` the max number of fragments to process before returning"]
    #[doc = " \n# Return\n >= 0 indicates the number of fragments processed, -1 for failure"]
    pub fn poll<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
        fragment_limit: ::std::os::raw::c_int,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_replay_merge_poll(
                self.get_inner(),
                {
                    let callback: aeron_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the image used for the merging replay and live stream."]
    #[doc = " The aeron_archive_replay_merge_do_work will be called before the poll so that processing of the merge can be done."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` the handler to call for incoming fragments"]
    #[doc = " \n - `clientd` the clientd to provide to the handler"]
    #[doc = " \n - `fragment_limit` the max number of fragments to process before returning"]
    #[doc = " \n# Return\n >= 0 indicates the number of fragments processed, -1 for failure"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn poll_once<AeronFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> ()>(
        &self,
        mut handler: AeronFragmentHandlerHandlerImpl,
        fragment_limit: ::std::os::raw::c_int,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_replay_merge_poll(
                self.get_inner(),
                Some(
                    aeron_fragment_handler_t_callback_for_once_closure::<
                        AeronFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "The image used for the replay and live stream."]
    #[doc = ""]
    #[doc = " \n# Return\n the `AeronImage`"]
    pub fn image(&self) -> AeronImage {
        unsafe {
            let result = aeron_archive_replay_merge_image(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Is the live stream merged and the replay stopped?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if merged, false otherwise"]
    pub fn is_merged(&self) -> bool {
        unsafe {
            let result = aeron_archive_replay_merge_is_merged(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Has the replay_merge failed due to an error?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if an error occurred"]
    pub fn has_failed(&self) -> bool {
        unsafe {
            let result = aeron_archive_replay_merge_has_failed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Is the live destination added to the subscription?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if the live destination is added to the subscription"]
    pub fn is_live_added(&self) -> bool {
        unsafe {
            let result = aeron_archive_replay_merge_is_live_added(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_replay_merge_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_replay_merge_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_replay_merge_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveReplayMerge {
    type Target = aeron_archive_replay_merge_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_replay_merge_t> for AeronArchiveReplayMerge {
    #[inline]
    fn from(value: *mut aeron_archive_replay_merge_t) -> Self {
        AeronArchiveReplayMerge {
            inner: CResource::Borrowed(value),
            _subscription: None,
            _aeron_archive: None,
        }
    }
}
impl From<AeronArchiveReplayMerge> for *mut aeron_archive_replay_merge_t {
    #[inline]
    fn from(value: AeronArchiveReplayMerge) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveReplayMerge> for *mut aeron_archive_replay_merge_t {
    #[inline]
    fn from(value: &AeronArchiveReplayMerge) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveReplayMerge> for aeron_archive_replay_merge_t {
    #[inline]
    fn from(value: AeronArchiveReplayMerge) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_replay_merge_t> for AeronArchiveReplayMerge {
    #[inline]
    fn from(value: *const aeron_archive_replay_merge_t) -> Self {
        AeronArchiveReplayMerge {
            inner: CResource::Borrowed(value as *mut aeron_archive_replay_merge_t),
            _subscription: None,
            _aeron_archive: None,
        }
    }
}
impl From<aeron_archive_replay_merge_t> for AeronArchiveReplayMerge {
    #[inline]
    fn from(value: aeron_archive_replay_merge_t) -> Self {
        AeronArchiveReplayMerge {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
            _subscription: None,
            _aeron_archive: None,
        }
    }
}
#[doc = "Struct containing the available replay parameters."]
#[derive(Clone)]
pub struct AeronArchiveReplayParams {
    inner: CResource<aeron_archive_replay_params_t>,
}
impl core::fmt::Debug for AeronArchiveReplayParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveReplayParams))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveReplayParams))
                .field("inner", &self.inner)
                .field(
                    stringify!(bounding_limit_counter_id),
                    &self.bounding_limit_counter_id(),
                )
                .field(stringify!(file_io_max_length), &self.file_io_max_length())
                .field(stringify!(position), &self.position())
                .field(stringify!(length), &self.length())
                .field(stringify!(replay_token), &self.replay_token())
                .field(
                    stringify!(subscription_registration_id),
                    &self.subscription_registration_id(),
                )
                .finish()
        }
    }
}
impl AeronArchiveReplayParams {
    #[inline]
    pub fn new(
        bounding_limit_counter_id: i32,
        file_io_max_length: i32,
        position: i64,
        length: i64,
        replay_token: i64,
        subscription_registration_id: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_replay_params_t {
                    bounding_limit_counter_id: bounding_limit_counter_id.into(),
                    file_io_max_length: file_io_max_length.into(),
                    position: position.into(),
                    length: length.into(),
                    replay_token: replay_token.into(),
                    subscription_registration_id: subscription_registration_id.into(),
                };
                let inner_ptr: *mut aeron_archive_replay_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_replay_params_t)
                );
                let inst: aeron_archive_replay_params_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_replay_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_replay_params_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn bounding_limit_counter_id(&self) -> i32 {
        self.bounding_limit_counter_id.into()
    }
    #[inline]
    pub fn file_io_max_length(&self) -> i32 {
        self.file_io_max_length.into()
    }
    #[inline]
    pub fn position(&self) -> i64 {
        self.position.into()
    }
    #[inline]
    pub fn length(&self) -> i64 {
        self.length.into()
    }
    #[inline]
    pub fn replay_token(&self) -> i64 {
        self.replay_token.into()
    }
    #[inline]
    pub fn subscription_registration_id(&self) -> i64 {
        self.subscription_registration_id.into()
    }
    #[inline]
    #[doc = "Initialize an `AeronArchiveReplayParams` with the default values."]
    pub fn init(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_replay_params_init(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_replay_params_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_replay_params_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_replay_params_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveReplayParams {
    type Target = aeron_archive_replay_params_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_replay_params_t> for AeronArchiveReplayParams {
    #[inline]
    fn from(value: *mut aeron_archive_replay_params_t) -> Self {
        AeronArchiveReplayParams {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveReplayParams> for *mut aeron_archive_replay_params_t {
    #[inline]
    fn from(value: AeronArchiveReplayParams) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveReplayParams> for *mut aeron_archive_replay_params_t {
    #[inline]
    fn from(value: &AeronArchiveReplayParams) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveReplayParams> for aeron_archive_replay_params_t {
    #[inline]
    fn from(value: AeronArchiveReplayParams) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_replay_params_t> for AeronArchiveReplayParams {
    #[inline]
    fn from(value: *const aeron_archive_replay_params_t) -> Self {
        AeronArchiveReplayParams {
            inner: CResource::Borrowed(value as *mut aeron_archive_replay_params_t),
        }
    }
}
impl From<aeron_archive_replay_params_t> for AeronArchiveReplayParams {
    #[inline]
    fn from(value: aeron_archive_replay_params_t) -> Self {
        AeronArchiveReplayParams {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveReplayParams {
    fn default() -> Self {
        AeronArchiveReplayParams::new_zeroed_on_heap()
    }
}
impl AeronArchiveReplayParams {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[doc = "Struct containing the available replication parameters."]
#[derive(Clone)]
pub struct AeronArchiveReplicationParams {
    inner: CResource<aeron_archive_replication_params_t>,
}
impl core::fmt::Debug for AeronArchiveReplicationParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchiveReplicationParams))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchiveReplicationParams))
                .field("inner", &self.inner)
                .field(stringify!(stop_position), &self.stop_position())
                .field(stringify!(dst_recording_id), &self.dst_recording_id())
                .field(stringify!(channel_tag_id), &self.channel_tag_id())
                .field(stringify!(subscription_tag_id), &self.subscription_tag_id())
                .field(stringify!(file_io_max_length), &self.file_io_max_length())
                .field(
                    stringify!(replication_session_id),
                    &self.replication_session_id(),
                )
                .finish()
        }
    }
}
impl AeronArchiveReplicationParams {
    #[inline]
    pub fn new(
        stop_position: i64,
        dst_recording_id: i64,
        live_destination: &std::ffi::CStr,
        replication_channel: &std::ffi::CStr,
        src_response_channel: &std::ffi::CStr,
        channel_tag_id: i64,
        subscription_tag_id: i64,
        file_io_max_length: i32,
        replication_session_id: i32,
        encoded_credentials: &AeronArchiveEncodedCredentials,
    ) -> Result<Self, AeronCError> {
        let encoded_credentials_copy = encoded_credentials.clone();
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_archive_replication_params_t {
                    stop_position: stop_position.into(),
                    dst_recording_id: dst_recording_id.into(),
                    live_destination: live_destination.as_ptr(),
                    replication_channel: replication_channel.as_ptr(),
                    src_response_channel: src_response_channel.as_ptr(),
                    channel_tag_id: channel_tag_id.into(),
                    subscription_tag_id: subscription_tag_id.into(),
                    file_io_max_length: file_io_max_length.into(),
                    replication_session_id: replication_session_id.into(),
                    encoded_credentials: encoded_credentials.into(),
                };
                let inner_ptr: *mut aeron_archive_replication_params_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_replication_params_t)
                );
                let inst: aeron_archive_replication_params_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_replication_params_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_replication_params_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn stop_position(&self) -> i64 {
        self.stop_position.into()
    }
    #[inline]
    pub fn dst_recording_id(&self) -> i64 {
        self.dst_recording_id.into()
    }
    #[inline]
    pub fn live_destination(&self) -> &str {
        if self.live_destination.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.live_destination)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn replication_channel(&self) -> &str {
        if self.replication_channel.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.replication_channel)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn src_response_channel(&self) -> &str {
        if self.src_response_channel.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.src_response_channel)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn channel_tag_id(&self) -> i64 {
        self.channel_tag_id.into()
    }
    #[inline]
    pub fn subscription_tag_id(&self) -> i64 {
        self.subscription_tag_id.into()
    }
    #[inline]
    pub fn file_io_max_length(&self) -> i32 {
        self.file_io_max_length.into()
    }
    #[inline]
    pub fn replication_session_id(&self) -> i32 {
        self.replication_session_id.into()
    }
    #[inline]
    pub fn encoded_credentials(&self) -> AeronArchiveEncodedCredentials {
        self.encoded_credentials.into()
    }
    #[inline]
    #[doc = "Initialize an `AeronArchiveReplicationParams` with the default values"]
    pub fn init(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_replication_params_init(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_replication_params_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_replication_params_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_replication_params_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchiveReplicationParams {
    type Target = aeron_archive_replication_params_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_replication_params_t> for AeronArchiveReplicationParams {
    #[inline]
    fn from(value: *mut aeron_archive_replication_params_t) -> Self {
        AeronArchiveReplicationParams {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchiveReplicationParams> for *mut aeron_archive_replication_params_t {
    #[inline]
    fn from(value: AeronArchiveReplicationParams) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchiveReplicationParams> for *mut aeron_archive_replication_params_t {
    #[inline]
    fn from(value: &AeronArchiveReplicationParams) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchiveReplicationParams> for aeron_archive_replication_params_t {
    #[inline]
    fn from(value: AeronArchiveReplicationParams) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_replication_params_t> for AeronArchiveReplicationParams {
    #[inline]
    fn from(value: *const aeron_archive_replication_params_t) -> Self {
        AeronArchiveReplicationParams {
            inner: CResource::Borrowed(value as *mut aeron_archive_replication_params_t),
        }
    }
}
impl From<aeron_archive_replication_params_t> for AeronArchiveReplicationParams {
    #[inline]
    fn from(value: aeron_archive_replication_params_t) -> Self {
        AeronArchiveReplicationParams {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronArchiveReplicationParams {
    fn default() -> Self {
        AeronArchiveReplicationParams::new_zeroed_on_heap()
    }
}
impl AeronArchiveReplicationParams {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronArchive {
    inner: CResource<aeron_archive_t>,
}
impl core::fmt::Debug for AeronArchive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronArchive))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronArchive))
                .field("inner", &self.inner)
                .field(stringify!(owns_ctx), &self.owns_ctx())
                .field(
                    stringify!(owns_control_response_subscription),
                    &self.owns_control_response_subscription(),
                )
                .field(stringify!(archive_id), &self.archive_id())
                .field(stringify!(is_in_callback), &self.is_in_callback())
                .finish()
        }
    }
}
impl AeronArchive {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_archive_t)
                );
                let inst: aeron_archive_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_archive_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_archive_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn owns_ctx(&self) -> bool {
        self.owns_ctx.into()
    }
    #[inline]
    pub fn ctx(&self) -> AeronArchiveContext {
        self.ctx.into()
    }
    #[inline]
    pub fn lock(&self) -> aeron_mutex_t {
        self.lock.into()
    }
    #[inline]
    pub fn archive_proxy(&self) -> AeronArchiveProxy {
        self.archive_proxy.into()
    }
    #[inline]
    pub fn owns_control_response_subscription(&self) -> bool {
        self.owns_control_response_subscription.into()
    }
    #[inline]
    pub fn subscription(&self) -> AeronSubscription {
        self.subscription.into()
    }
    #[inline]
    pub fn recording_descriptor_poller(&self) -> AeronArchiveRecordingDescriptorPoller {
        self.recording_descriptor_poller.into()
    }
    #[inline]
    pub fn recording_subscription_descriptor_poller(
        &self,
    ) -> AeronArchiveRecordingSubscriptionDescriptorPoller {
        self.recording_subscription_descriptor_poller.into()
    }
    #[inline]
    pub fn archive_id(&self) -> i64 {
        self.archive_id.into()
    }
    #[inline]
    pub fn is_in_callback(&self) -> bool {
        self.is_in_callback.into()
    }
    #[inline]
    #[doc = "Close the connection to the Aeron Archive and free up associated resources."]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_archive_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Retrieve the underlying `AeronArchiveContext` used to configure the provided `AeronArchive`."]
    pub fn get_archive_context(&self) -> AeronArchiveContext {
        unsafe {
            let result = aeron_archive_get_archive_context(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Retrieve the underlying `AeronArchiveContext` used to configure the provided `AeronArchive`."]
    #[doc = " \n"]
    #[doc = " Additionally, calling this function transfers ownership of the returned `AeronArchiveContext` to the caller."]
    #[doc = " i.e. it is now the the caller's responsibility to close the context."]
    #[doc = " This is useful when wrapping the C library in other, higher level languages."]
    pub fn get_and_own_archive_context(&self) -> AeronArchiveContext {
        unsafe {
            let result = aeron_archive_get_and_own_archive_context(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Retrieve the archive id of the connected Aeron Archive."]
    pub fn get_archive_id(&self) -> i64 {
        unsafe {
            let result = aeron_archive_get_archive_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Retrieve the underlying `AeronSubscription` used for reading responses from the connected Aeron Archive."]
    pub fn get_control_response_subscription(&self) -> AeronSubscription {
        unsafe {
            let result = aeron_archive_get_control_response_subscription(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Retrieve the underlying `AeronSubscription` used for reading responses from the connected Aeron Archive."]
    #[doc = " \n"]
    #[doc = " Additionally, calling this function transfers ownership of the returned `AeronSubscription` to the caller."]
    #[doc = " i.e. it is now the caller's responsibility to close the subscription."]
    #[doc = " This is useful when wrapping the C library in other, high level languages."]
    pub fn get_and_own_control_response_subscription(&self) -> AeronSubscription {
        unsafe {
            let result = aeron_archive_get_and_own_control_response_subscription(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn control_session_id(&self) -> i64 {
        unsafe {
            let result = aeron_archive_control_session_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Poll for recording signals, dispatching them to the configured aeron_archive_recording_signal_consumer_func_t in the context"]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param that indicates the number of recording signals dispatched."]
    pub fn poll_for_recording_signals(&self) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code =
                aeron_archive_poll_for_recording_signals(&mut mut_result, self.get_inner());
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the response stream once for an error."]
    #[doc = " If another message is present then it will be skipped over, so only call when not expecting another response."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 if an error sent from the Aeron Archive is found, in which case, the provided buffer contains the error message."]
    #[doc = " If there was no error, the buffer will be an empty string."]
    #[doc = " \n"]
    #[doc = " -1 if an error occurs while attempting to read from the subscription."]
    pub fn poll_for_error_response(
        &self,
        buffer: *mut ::std::os::raw::c_char,
        buffer_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_poll_for_error_response(
                self.get_inner(),
                buffer.into(),
                buffer_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the response stream once for an error."]
    #[doc = " If another message is present then it will be skipped over, so only call when not expecting another response."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 if an error sent from the Aeron Archive is found, in which case, the provided buffer contains the error message."]
    #[doc = " If there was no error, the buffer will be an empty string."]
    #[doc = " \n"]
    #[doc = " -1 if an error occurs while attempting to read from the subscription."]
    pub fn poll_for_error_response_as_string(
        &self,
        max_length: usize,
    ) -> Result<String, AeronCError> {
        let mut result = String::with_capacity(max_length);
        self.poll_for_error_response_into(&mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Poll the response stream once for an error."]
    #[doc = " If another message is present then it will be skipped over, so only call when not expecting another response."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 if an error sent from the Aeron Archive is found, in which case, the provided buffer contains the error message."]
    #[doc = " If there was no error, the buffer will be an empty string."]
    #[doc = " \n"]
    #[doc = " -1 if an error occurs while attempting to read from the subscription."]
    #[doc = "NOTE: allocation friendly method, the string capacity must be set as it will truncate string to capacity it will never grow the string. So if you pass String::new() it will write 0 chars"]
    pub fn poll_for_error_response_into(
        &self,
        dst_truncate_to_capacity: &mut String,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let capacity = dst_truncate_to_capacity.capacity();
            let vec = dst_truncate_to_capacity.as_mut_vec();
            vec.set_len(capacity);
            let result = self.poll_for_error_response(vec.as_mut_ptr() as *mut _, capacity)?;
            let mut len = 0;
            loop {
                if len == capacity {
                    break;
                }
                let val = vec[len];
                if val == 0 {
                    break;
                }
                len += 1;
            }
            vec.set_len(len);
            Ok(result)
        }
    }
    #[inline]
    #[doc = "Poll the response stream once for an error."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 if no error is found OR if an error is found but an error handler is specified in the context."]
    #[doc = " \n"]
    #[doc = " -1 if an error is found and no error handler is specified.  The error message can be retrieved by calling aeron_errmsg()"]
    pub fn check_for_error_response(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_check_for_error_response(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Start recording a channel/stream pairing."]
    #[doc = " \n"]
    #[doc = " Channels that include session id parameters are considered different than channels without session ids."]
    #[doc = " If a publication matches both a session id specific channel recording and a non session id specific recording,"]
    #[doc = " it will be recorded twice."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`subscription_id_p` out param set to the subscription id of the recording"]
    #[doc = " \n # Parameters
- `recording_channel` the channel of the publication to be recorded"]
    #[doc = " \n - `recording_stream_id` the stream id of the publication to be recorded"]
    #[doc = " \n - `source_location` the source location of the publication to be recorded"]
    #[doc = " \n - `auto_stop` should the recording be automatically stopped when complete"]
    pub fn start_recording(
        &self,
        recording_channel: &std::ffi::CStr,
        recording_stream_id: i32,
        source_location: aeron_archive_source_location_t,
        auto_stop: bool,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_start_recording(
                &mut mut_result,
                self.get_inner(),
                recording_channel.as_ptr(),
                recording_stream_id.into(),
                source_location.into(),
                auto_stop.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Fetch the position recorded for the specified recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`recording_position_p` out param set to the recording position of the specified recording"]
    #[doc = " \n # Parameters
- `recording_id` the active recording id"]
    pub fn get_recording_position(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_get_recording_position(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Fetch the start position for the specified recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`start_position_p` out param set to the start position of the specified recording"]
    #[doc = " \n # Parameters
- `recording_id` the active recording id"]
    pub fn get_start_position(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_get_start_position(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Fetch the stop position for the specified recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`stop_position_p` out param set to the stop position of the specified recording"]
    #[doc = " \n # Parameters
- `recording_id` the active recording id"]
    pub fn get_stop_position(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_get_stop_position(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Fetch the stop or active position for the specified recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`max_recorded_position_p` out param set to the stop or active position of the specified recording"]
    #[doc = " \n # Parameters
- `recording_id` the active recording id"]
    pub fn get_max_recorded_position(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_get_max_recorded_position(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop recording for the specified subscription id."]
    #[doc = " This is the subscription id returned from aeron_archive_start_recording or aeron_archive_extend_recording."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `subscription_id` the subscription id for the recording in the Aeron Archive"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_recording_subscription(&self, subscription_id: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_archive_stop_recording_subscription(self.get_inner(), subscription_id.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Try to stop a recording for the specified subscription id."]
    #[doc = " This is the subscription id returned from aeron_archive_start_recording or aeron_archive_extend_recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`stopped_p` out param indicating true if stopped, or false if the subscription is not currently active"]
    #[doc = " \n # Parameters
- `subscription_id` the subscription id for the recording in the Aeron Archive"]
    pub fn try_stop_recording_subscription(
        &self,
        subscription_id: i64,
    ) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_archive_try_stop_recording_subscription(
                &mut mut_result,
                self.get_inner(),
                subscription_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop recording for the specified channel and stream."]
    #[doc = " \n"]
    #[doc = " Channels that include session id parameters are considered different than channels without session ids."]
    #[doc = " Stopping a recording on a channel without a session id parameter will not stop the recording of any"]
    #[doc = " session id specific recordings that use the same channel and stream id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `channel` the channel of the recording to be stopped"]
    #[doc = " \n - `stream_id` the stream id of the recording to be stopped"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_recording_channel_and_stream(
        &self,
        channel: &std::ffi::CStr,
        stream_id: i32,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_stop_recording_channel_and_stream(
                self.get_inner(),
                channel.as_ptr(),
                stream_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Try to stop recording for the specified channel and stream."]
    #[doc = " \n"]
    #[doc = " Channels that include session id parameters are considered different than channels without session ids."]
    #[doc = " Stopping a recording on a channel without a session id parameter will not stop the recording of any"]
    #[doc = " session id specific recordings that use the same channel and stream id."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`stopped_p` out param indicating true if stopped, or false if the channel/stream pair is not currently active"]
    #[doc = " \n # Parameters
- `channel` the channel of the recording to be stopped"]
    #[doc = " \n - `stream_id` the stream id of the recording to be stopped"]
    pub fn try_stop_recording_channel_and_stream(
        &self,
        channel: &std::ffi::CStr,
        stream_id: i32,
    ) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_archive_try_stop_recording_channel_and_stream(
                &mut mut_result,
                self.get_inner(),
                channel.as_ptr(),
                stream_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop recording for the specified recording id."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`stopped_p` out param indicating true if stopped, or false if the recording is not currently active"]
    #[doc = " \n # Parameters
- `recording_id` the id of the recording to be stopped"]
    pub fn try_stop_recording_by_identity(&self, recording_id: i64) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_archive_try_stop_recording_by_identity(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop recording a session id specific recording that pertains to the given publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `publication` the publication to stop recording"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_recording_publication(
        &self,
        publication: &AeronPublication,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_archive_stop_recording_publication(self.get_inner(), publication.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Stop recording a session id specific recording that pertains to the given exclusive publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `exclusive_publication` the exclusive publication to stop recording"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_recording_exclusive_publication(
        &self,
        exclusive_publication: &AeronExclusivePublication,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_stop_recording_exclusive_publication(
                self.get_inner(),
                exclusive_publication.get_inner(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Find the last recording that matches the given criteria."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`recording_id_p` out param for the recording id that matches"]
    #[doc = " \n # Parameters
- `min_recording_id` the lowest recording id to search back to"]
    #[doc = " \n - `channel_fragment` for a 'contains' match on the original channel stored with the Aeron Archive"]
    #[doc = " \n - `stream_id` the stream id of the recording"]
    #[doc = " \n - `session_id` the session id of the recording"]
    pub fn find_last_matching_recording(
        &self,
        min_recording_id: i64,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        session_id: i32,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_find_last_matching_recording(
                &mut mut_result,
                self.get_inner(),
                min_recording_id.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                session_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "List a recording descriptor for a single recording id."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param indicating the number of descriptors found"]
    #[doc = " \n # Parameters
- `recording_id` the id of the recording"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    pub fn list_recording<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingDescriptorConsumerFuncCallback,
    >(
        &self,
        recording_id: i64,
        recording_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_archive_list_recording(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
                {
                    let callback: aeron_archive_recording_descriptor_consumer_func_t =
                        if recording_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_recording_descriptor_consumer_func_t_callback::<
                                    AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                recording_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "List a recording descriptor for a single recording id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `count_p` out param indicating the number of descriptors found"]
    #[doc = " \n - `recording_id` the id of the recording"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn list_recording_once<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingDescriptor) -> (),
    >(
        &self,
        count_p: &mut i32,
        recording_id: i64,
        mut recording_descriptor_consumer: AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_list_recording(
                count_p as *mut _,
                self.get_inner(),
                recording_id.into(),
                Some(
                    aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure::<
                        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                    >,
                ),
                &mut recording_descriptor_consumer as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "List all recording descriptors starting at a particular recording id, with a limit of total descriptors delivered."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param indicating the number of descriptors found"]
    #[doc = " \n # Parameters
- `from_recording_id` the id at which to begin the listing"]
    #[doc = " \n - `record_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    pub fn list_recordings<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingDescriptorConsumerFuncCallback,
    >(
        &self,
        from_recording_id: i64,
        record_count: i32,
        recording_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_archive_list_recordings(
                &mut mut_result,
                self.get_inner(),
                from_recording_id.into(),
                record_count.into(),
                {
                    let callback: aeron_archive_recording_descriptor_consumer_func_t =
                        if recording_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_recording_descriptor_consumer_func_t_callback::<
                                    AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                recording_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "List all recording descriptors starting at a particular recording id, with a limit of total descriptors delivered."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `count_p` out param indicating the number of descriptors found"]
    #[doc = " \n - `from_recording_id` the id at which to begin the listing"]
    #[doc = " \n - `record_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn list_recordings_once<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingDescriptor) -> (),
    >(
        &self,
        count_p: &mut i32,
        from_recording_id: i64,
        record_count: i32,
        mut recording_descriptor_consumer: AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_list_recordings(
                count_p as *mut _,
                self.get_inner(),
                from_recording_id.into(),
                record_count.into(),
                Some(
                    aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure::<
                        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                    >,
                ),
                &mut recording_descriptor_consumer as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "List all recording descriptors for a given channel fragment and stream id, starting at a particular recording id, with a limit of total descriptors delivered."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param indicating the number of descriptors found"]
    #[doc = " \n # Parameters
- `from_recording_id` the id at which to begin the listing"]
    #[doc = " \n - `record_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `channel_fragment` for a 'contains' match on the original channel stored with the Aeron Archive"]
    #[doc = " \n - `stream_id` the stream id of the recording"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    pub fn list_recordings_for_uri<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingDescriptorConsumerFuncCallback,
    >(
        &self,
        from_recording_id: i64,
        record_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        recording_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_archive_list_recordings_for_uri(
                &mut mut_result,
                self.get_inner(),
                from_recording_id.into(),
                record_count.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                {
                    let callback: aeron_archive_recording_descriptor_consumer_func_t =
                        if recording_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some(
                                aeron_archive_recording_descriptor_consumer_func_t_callback::<
                                    AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                recording_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "List all recording descriptors for a given channel fragment and stream id, starting at a particular recording id, with a limit of total descriptors delivered."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `count_p` out param indicating the number of descriptors found"]
    #[doc = " \n - `from_recording_id` the id at which to begin the listing"]
    #[doc = " \n - `record_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `channel_fragment` for a 'contains' match on the original channel stored with the Aeron Archive"]
    #[doc = " \n - `stream_id` the stream id of the recording"]
    #[doc = " \n - `recording_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_descriptor_consumer_clientd` to be passed for each descriptor"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn list_recordings_for_uri_once<
        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingDescriptor) -> (),
    >(
        &self,
        count_p: &mut i32,
        from_recording_id: i64,
        record_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        mut recording_descriptor_consumer: AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_list_recordings_for_uri(
                count_p as *mut _,
                self.get_inner(),
                from_recording_id.into(),
                record_count.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                Some(
                    aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure::<
                        AeronArchiveRecordingDescriptorConsumerFuncHandlerImpl,
                    >,
                ),
                &mut recording_descriptor_consumer as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Start a replay"]
    #[doc = " \n"]
    #[doc = " The lower 32-bits of the replay session id contain the session id of the image of the received replay"]
    #[doc = " and can be obtained by casting the replay session id to an int32_t."]
    #[doc = " All 64-bits are required to uniquely identify the replay when calling aeron_archive_stop_replay."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`replay_session_id_p` out param set to the replay session id"]
    #[doc = " \n # Parameters
- `recording_id` the id of the recording"]
    #[doc = " \n - `replay_channel` the channel to which the replay should be sent"]
    #[doc = " \n - `replay_stream_id` the stream id to which the replay should be sent"]
    #[doc = " \n - `params` the `AeronArchiveReplayParams` that control the behaviour of the replay"]
    pub fn start_replay(
        &self,
        recording_id: i64,
        replay_channel: &std::ffi::CStr,
        replay_stream_id: i32,
        params: &AeronArchiveReplayParams,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_start_replay(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
                replay_channel.as_ptr(),
                replay_stream_id.into(),
                params.get_inner(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Truncate a stopped recording to the specified position."]
    #[doc = " The position must be less than the stopped position."]
    #[doc = " The position must be on a fragment boundary."]
    #[doc = " Truncating a recording to the start position effectively deletes the recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the number of segments deleted"]
    #[doc = " \n # Parameters
- `recording_id` the id of the recording"]
    #[doc = " \n - `position` the position to which the recording will be truncated"]
    pub fn truncate_recording(&self, recording_id: i64, position: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_truncate_recording(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
                position.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop a replay session."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `replay_session_id` the replay session id indicating the replay to stop"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_replay(&self, replay_session_id: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_stop_replay(self.get_inner(), replay_session_id.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Stop all replays matching a recording id."]
    #[doc = " If recording_id is AERON_NULL_VALUE then match all replays."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `recording_id` the id of the recording for which all replays will be stopped"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_all_replays(&self, recording_id: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_stop_all_replays(self.get_inner(), recording_id.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "List active recording subscriptions in the Aeron Archive."]
    #[doc = " These are the result of calling aeron_archive_start_recording or aeron_archive_extend_recording."]
    #[doc = " The subscription id in the returned descriptor can be used when calling aeron_archive_stop_recording_subscription."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the count of matched subscriptions"]
    #[doc = " \n # Parameters
- `pseudo_index` the index into the active list at which to begin listing"]
    #[doc = " \n - `subscription_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `channel_fragment` for a 'contains' match on the original channel stored with the Aeron Archive"]
    #[doc = " \n - `stream_id` the stream id of the recording"]
    #[doc = " \n - `apply_stream_id` whether or not the stream id should be matched"]
    #[doc = " \n - `recording_subscription_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_subscription_descriptor_consumer_clientd` to be passed for each descriptor"]
    pub fn list_recording_subscriptions<
        AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl: AeronArchiveRecordingSubscriptionDescriptorConsumerFuncCallback,
    >(
        &self,
        pseudo_index: i32,
        subscription_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        apply_stream_id: bool,
        recording_subscription_descriptor_consumer: Option<
            &Handler<AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl>,
        >,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_archive_list_recording_subscriptions(
                &mut mut_result,
                self.get_inner(),
                pseudo_index.into(),
                subscription_count.into(),
                channel_fragment.as_ptr(),
                stream_id.into(),
                apply_stream_id.into(),
                {
                    let callback: aeron_archive_recording_subscription_descriptor_consumer_func_t =
                        if recording_subscription_descriptor_consumer.is_none() {
                            None
                        } else {
                            Some (aeron_archive_recording_subscription_descriptor_consumer_func_t_callback :: < AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl >)
                        };
                    callback
                },
                recording_subscription_descriptor_consumer
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "List active recording subscriptions in the Aeron Archive."]
    #[doc = " These are the result of calling aeron_archive_start_recording or aeron_archive_extend_recording."]
    #[doc = " The subscription id in the returned descriptor can be used when calling aeron_archive_stop_recording_subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `count_p` out param set to the count of matched subscriptions"]
    #[doc = " \n - `pseudo_index` the index into the active list at which to begin listing"]
    #[doc = " \n - `subscription_count` the limit of total descriptors to deliver"]
    #[doc = " \n - `channel_fragment` for a 'contains' match on the original channel stored with the Aeron Archive"]
    #[doc = " \n - `stream_id` the stream id of the recording"]
    #[doc = " \n - `apply_stream_id` whether or not the stream id should be matched"]
    #[doc = " \n - `recording_subscription_descriptor_consumer` to be called for each descriptor"]
    #[doc = " \n - `recording_subscription_descriptor_consumer_clientd` to be passed for each descriptor"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn list_recording_subscriptions_once<
        AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl: FnMut(AeronArchiveRecordingSubscriptionDescriptor) -> (),
    >(
        &self,
        count_p: &mut i32,
        pseudo_index: i32,
        subscription_count: i32,
        channel_fragment: &std::ffi::CStr,
        stream_id: i32,
        apply_stream_id: bool,
        mut recording_subscription_descriptor_consumer : AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_list_recording_subscriptions (count_p as * mut _ , self . get_inner () , pseudo_index . into () , subscription_count . into () , channel_fragment . as_ptr () , stream_id . into () , apply_stream_id . into () , Some (aeron_archive_recording_subscription_descriptor_consumer_func_t_callback_for_once_closure :: < AeronArchiveRecordingSubscriptionDescriptorConsumerFuncHandlerImpl >) , & mut recording_subscription_descriptor_consumer as * mut _ as * mut std :: os :: raw :: c_void) ;
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Purge a stopped recording."]
    #[doc = " i.e. Mark the recording as INVALID at the Archive and delete the corresponding segment files."]
    #[doc = " The space in the Catalog will be reclaimed upon compaction."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`deleted_segments_count_p` out param set to the number of deleted segments"]
    #[doc = " \n # Parameters
- `recording_id` the id of the stopped recording to be purged"]
    pub fn purge_recording(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_purge_recording(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Extend an existing, non-active recording for a channel and stream pairing."]
    #[doc = " \n"]
    #[doc = " The channel must be configured with the initial position from which it will be extended."]
    #[doc = " This can be done with aeron_uri_string_builder_set_initial_position."]
    #[doc = " The details required to initialize can be found by calling aeron_archive_list_recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`subscription_id_p` out param set to the subscription id of the recording"]
    #[doc = " \n # Parameters
- `recording_id` the id of the existing recording"]
    #[doc = " \n - `recording_channel` the channel of the publication to be recorded"]
    #[doc = " \n - `recording_stream_id` the stream id of the publication to be recorded"]
    #[doc = " \n - `source_location` the source location of the publication to be recorded"]
    #[doc = " \n - `auto_stop` should the recording be automatically stopped when complete"]
    pub fn extend_recording(
        &self,
        recording_id: i64,
        recording_channel: &std::ffi::CStr,
        recording_stream_id: i32,
        source_location: aeron_archive_source_location_t,
        auto_stop: bool,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_extend_recording(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
                recording_channel.as_ptr(),
                recording_stream_id.into(),
                source_location.into(),
                auto_stop.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Replicate a recording from a source Archive to a destination."]
    #[doc = " This can be considered a backup for a primary Archive."]
    #[doc = " The source recording will be replayed via the provided replay channel and use the original stream id."]
    #[doc = " The behavior of the replication will be governed by the values specified in the `AeronArchiveReplicationParams`."]
    #[doc = " \n"]
    #[doc = " For a source recording that is still active, the replay can merge with the live stream and then follow it directly and no longer require the replay from the source."]
    #[doc = " This would require a multicast live destination."]
    #[doc = " \n"]
    #[doc = " Errors will be reported asynchronously and can be checked for with aeron_archive_check_for_error_response and aeron_archive_poll_for_error_response."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`replication_id_p` out param set to the replication id that can be used to stop the replication"]
    #[doc = " \n # Parameters
- `src_recording_id` the recording id that must exist at the source Archive"]
    #[doc = " \n - `src_control_channel` remote control channel for the source archive on which to instruct the replay"]
    #[doc = " \n - `src_control_stream_id` remote control stream id for the source archive on which to instruct the replay"]
    #[doc = " \n - `params` optional parameters to configure the behavior of the replication"]
    pub fn replicate(
        &self,
        src_recording_id: i64,
        src_control_channel: &std::ffi::CStr,
        src_control_stream_id: i32,
        params: &AeronArchiveReplicationParams,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_replicate(
                &mut mut_result,
                self.get_inner(),
                src_recording_id.into(),
                src_control_channel.as_ptr(),
                src_control_stream_id.into(),
                params.get_inner(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Stop a replication by the replication id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `replication_id` the replication id retrieved when calling aeron_archive_replicate"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn stop_replication(&self, replication_id: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_stop_replication(self.get_inner(), replication_id.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Try to stop a replication by the replication id."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`stopped_p` out param indicating true if stopped, or false if the recording is not currently active"]
    #[doc = " \n # Parameters
- `replication_id` the replication id retrieved when calling aeron_archive_replicate"]
    pub fn try_stop_replication(&self, replication_id: i64) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_archive_try_stop_replication(
                &mut mut_result,
                self.get_inner(),
                replication_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Detach segments from the beginning of a recording up to the provided new start position."]
    #[doc = " \n"]
    #[doc = " The new start position must be the first byte position of a segment after the existing start position."]
    #[doc = " \n"]
    #[doc = " It is not possible to detach segments which are active for recording or being replayed."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `recording_id` the id of an existing recording"]
    #[doc = " \n - `new_start_position` the new starting position for the recording after the segments are detached"]
    #[doc = " \n# Return\n 0 for success, -1 for failure"]
    pub fn detach_segments(
        &self,
        recording_id: i64,
        new_start_position: i64,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_archive_detach_segments(
                self.get_inner(),
                recording_id.into(),
                new_start_position.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Delete segments which have been previously detached from a recording."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the number of segments deleted"]
    #[doc = " \n # Parameters
- `recording_id` the id of an existing recording"]
    pub fn delete_detached_segments(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_delete_detached_segments(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Purge (Detach and delete) segments from the beginning of a recording up to the provided new start position."]
    #[doc = " \n"]
    #[doc = " The new start position must be the first byte position of a segment after the existing start position."]
    #[doc = " \n"]
    #[doc = " It is not possible to detach segments which are active for recording or being replayed."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the number of segments deleted"]
    #[doc = " \n # Parameters
- `recording_id` the id of an existing recording"]
    #[doc = " \n - `new_start_position` the new starting position for the recording after the segments are detached"]
    pub fn purge_segments(
        &self,
        recording_id: i64,
        new_start_position: i64,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_purge_segments(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
                new_start_position.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Attach segments to the beginning of a recording to restore history that was previously detached."]
    #[doc = " \n"]
    #[doc = " Segment files must match the existing recording and join exactly to the start position of the recording they are being attached to."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the number of segments attached"]
    #[doc = " \n # Parameters
- `recording_id` the id of an existing recording"]
    pub fn attach_segments(&self, recording_id: i64) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_attach_segments(
                &mut mut_result,
                self.get_inner(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Migrate segments from a source recording and attach them to the beginning of a destination recording."]
    #[doc = " \n"]
    #[doc = " The source recording must match the destination recording for segment length, term length, mtu length,"]
    #[doc = " stream id, plus the stop position and term id of the source must join with the start position of the destination"]
    #[doc = " and be on a segment boundary."]
    #[doc = " \n"]
    #[doc = " The source recording will be effectively truncated back to its start position after the migration."]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`count_p` out param set to the number of segments deleted"]
    #[doc = " \n # Parameters
- `src_recording_id` the id of an existing recording from which segments will be migrated"]
    #[doc = " \n - `dst_recording_id` the id of an existing recording to which segments will be migrated"]
    pub fn migrate_segments(
        &self,
        src_recording_id: i64,
        dst_recording_id: i64,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_migrate_segments(
                &mut mut_result,
                self.get_inner(),
                src_recording_id.into(),
                dst_recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Position of the recorded stream at the base of a segment file."]
    #[doc = " \n"]
    #[doc = " If a recording starts within a term then the base position can be before the recording started."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `start_position` start position of the stream"]
    #[doc = " \n - `position` position in the stream to calculate the segment base position from."]
    #[doc = " \n - `term_buffer_length` term buffer length of the stream"]
    #[doc = " \n - `segment_file_length` segment file length, which is a multiple of term buffer length"]
    #[doc = " \n# Return\n the position of the recorded stream at the beginning of a segment file"]
    pub fn segment_file_base_position(
        start_position: i64,
        position: i64,
        term_buffer_length: i32,
        segment_file_length: i32,
    ) -> i64 {
        unsafe {
            let result = aeron_archive_segment_file_base_position(
                start_position.into(),
                position.into(),
                term_buffer_length.into(),
                segment_file_length.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn idle(&self) -> () {
        unsafe {
            let result = aeron_archive_idle(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn control_response_poller(&self) -> AeronArchiveControlResponsePoller {
        unsafe {
            let result = aeron_archive_control_response_poller(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn proxy(&self) -> AeronArchiveProxy {
        unsafe {
            let result = aeron_archive_proxy(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn next_correlation_id(&self) -> i64 {
        unsafe {
            let result = aeron_archive_next_correlation_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn poll_for_response(
        &self,
        operation_name: &std::ffi::CStr,
        correlation_id: i64,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_archive_poll_for_response(
                &mut mut_result,
                self.get_inner(),
                operation_name.as_ptr(),
                correlation_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_archive_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_archive_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_archive_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronArchive {
    type Target = aeron_archive_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_archive_t> for AeronArchive {
    #[inline]
    fn from(value: *mut aeron_archive_t) -> Self {
        AeronArchive {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronArchive> for *mut aeron_archive_t {
    #[inline]
    fn from(value: AeronArchive) -> Self {
        value.get_inner()
    }
}
impl From<&AeronArchive> for *mut aeron_archive_t {
    #[inline]
    fn from(value: &AeronArchive) -> Self {
        value.get_inner()
    }
}
impl From<AeronArchive> for aeron_archive_t {
    #[inline]
    fn from(value: AeronArchive) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_archive_t> for AeronArchive {
    #[inline]
    fn from(value: *const aeron_archive_t) -> Self {
        AeronArchive {
            inner: CResource::Borrowed(value as *mut aeron_archive_t),
        }
    }
}
impl From<aeron_archive_t> for AeronArchive {
    #[inline]
    fn from(value: aeron_archive_t) -> Self {
        AeronArchive {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronArchive {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronArchive));
                    let result = self.close();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronArchive));
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct AeronAsyncAddCounter {
    inner: CResource<aeron_async_add_counter_t>,
}
impl core::fmt::Debug for AeronAsyncAddCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncAddCounter))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncAddCounter))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncAddCounter {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_async_add_counter_t)
                );
                let inst: aeron_async_add_counter_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_async_add_counter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_async_add_counter_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Gets the registration id for addition of the counter. Note that using this after a call to poll the succeeds or"]
    #[doc = " errors is undefined behaviour. As the async_add_counter_t may have been freed."]
    #[doc = ""]
    #[doc = " \n# Return\n registration id for the counter."]
    pub fn get_registration_id(&self) -> i64 {
        unsafe {
            let result = aeron_async_add_counter_get_registration_id(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_add_counter_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_add_counter_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_add_counter_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncAddCounter {
    type Target = aeron_async_add_counter_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_add_counter_t> for AeronAsyncAddCounter {
    #[inline]
    fn from(value: *mut aeron_async_add_counter_t) -> Self {
        AeronAsyncAddCounter {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncAddCounter> for *mut aeron_async_add_counter_t {
    #[inline]
    fn from(value: AeronAsyncAddCounter) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncAddCounter> for *mut aeron_async_add_counter_t {
    #[inline]
    fn from(value: &AeronAsyncAddCounter) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncAddCounter> for aeron_async_add_counter_t {
    #[inline]
    fn from(value: AeronAsyncAddCounter) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_add_counter_t> for AeronAsyncAddCounter {
    #[inline]
    fn from(value: *const aeron_async_add_counter_t) -> Self {
        AeronAsyncAddCounter {
            inner: CResource::Borrowed(value as *mut aeron_async_add_counter_t),
        }
    }
}
impl From<aeron_async_add_counter_t> for AeronAsyncAddCounter {
    #[inline]
    fn from(value: aeron_async_add_counter_t) -> Self {
        AeronAsyncAddCounter {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl AeronCounter {
    #[inline]
    pub fn new(async_: &AeronAsyncAddCounter) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_async_add_counter_poll(ctx_field, async_.into()) },
            None,
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        })
    }
}
impl Aeron {
    #[inline]
    pub fn async_add_counter(
        &self,
        type_id: i32,
        key_buffer: &[u8],
        label_buffer: &str,
    ) -> Result<AeronAsyncAddCounter, AeronCError> {
        let mut result = AeronAsyncAddCounter::new(self, type_id, key_buffer, label_buffer);
        if let Ok(result) = &mut result {
            result.inner.add_dependency(self.clone());
        }
        result
    }
}
impl Aeron {
    #[inline]
    pub fn add_counter(
        &self,
        type_id: i32,
        key_buffer: &[u8],
        label_buffer: &str,
        timeout: std::time::Duration,
    ) -> Result<AeronCounter, AeronCError> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(poller) = AeronAsyncAddCounter::new(self, type_id, key_buffer, label_buffer) {
                while start.elapsed() <= timeout {
                    if let Some(result) = poller.poll()? {
                        return Ok(result);
                    }
                    #[cfg(debug_assertions)]
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            if start.elapsed() > timeout {
                log::error!("failed async poll for {:?}", self);
                return Err(AeronErrorType::TimedOut.into());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
impl AeronAsyncAddCounter {
    #[inline]
    pub fn new(
        client: &Aeron,
        type_id: i32,
        key_buffer: &[u8],
        label_buffer: &str,
    ) -> Result<Self, AeronCError> {
        let resource_async = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_async_add_counter(
                    ctx_field,
                    client.into(),
                    type_id.into(),
                    key_buffer.as_ptr() as *mut _,
                    key_buffer.len(),
                    label_buffer.as_ptr() as *const _,
                    label_buffer.len(),
                )
            },
            None,
            false,
            None,
        )?;
        let result = Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_async)),
        };
        result.inner.add_dependency(client.clone());
        Ok(result)
    }
    pub fn poll(&self) -> Result<Option<AeronCounter>, AeronCError> {
        let mut result = AeronCounter::new(self);
        if let Ok(result) = &mut result {
            unsafe {
                for d in (&mut *self.inner.as_owned().unwrap().dependencies.get()).iter_mut() {
                    result.inner.add_dependency(d.clone());
                }
                result.inner.as_owned().unwrap().auto_close.set(true);
            }
        }
        match result {
            Ok(result) => Ok(Some(result)),
            Err(AeronCError { code }) if code == 0 => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_blocking(&self, timeout: std::time::Duration) -> Result<AeronCounter, AeronCError> {
        if let Some(result) = self.poll()? {
            return Ok(result);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Some(result) = self.poll()? {
                return Ok(result);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {:?}", self);
        Err(AeronErrorType::TimedOut.into())
    }
}
#[derive(Clone)]
pub struct AeronAsyncAddExclusivePublication {
    inner: CResource<aeron_async_add_exclusive_publication_t>,
}
impl core::fmt::Debug for AeronAsyncAddExclusivePublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncAddExclusivePublication))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncAddExclusivePublication))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncAddExclusivePublication {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_async_add_exclusive_publication_t)
                );
                let inst: aeron_async_add_exclusive_publication_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_async_add_exclusive_publication_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_async_add_exclusive_publication_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Gets the registration id for addition of the exclusive_publication. Note that using this after a call to poll the"]
    #[doc = " succeeds or errors is undefined behaviour. As the async_add_exclusive_publication_t may have been freed."]
    #[doc = ""]
    #[doc = " \n# Return\n registration id for the exclusive_publication."]
    #[deprecated]
    #[doc = " @deprecated Use aeron_async_add_exclusive_publication_get_registration_id instead."]
    pub fn aeron_async_add_exclusive_exclusive_publication_get_registration_id(&self) -> i64 {
        unsafe {
            let result = aeron_async_add_exclusive_exclusive_publication_get_registration_id(
                self.get_inner(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Gets the registration id for addition of the exclusive_publication. Note that using this after a call to poll the"]
    #[doc = " succeeds or errors is undefined behaviour. As the async_add_exclusive_publication_t may have been freed."]
    #[doc = ""]
    #[doc = " \n# Return\n registration id for the exclusive_publication."]
    pub fn get_registration_id(&self) -> i64 {
        unsafe {
            let result =
                aeron_async_add_exclusive_publication_get_registration_id(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_add_exclusive_publication_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_add_exclusive_publication_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_add_exclusive_publication_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncAddExclusivePublication {
    type Target = aeron_async_add_exclusive_publication_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_add_exclusive_publication_t> for AeronAsyncAddExclusivePublication {
    #[inline]
    fn from(value: *mut aeron_async_add_exclusive_publication_t) -> Self {
        AeronAsyncAddExclusivePublication {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncAddExclusivePublication> for *mut aeron_async_add_exclusive_publication_t {
    #[inline]
    fn from(value: AeronAsyncAddExclusivePublication) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncAddExclusivePublication> for *mut aeron_async_add_exclusive_publication_t {
    #[inline]
    fn from(value: &AeronAsyncAddExclusivePublication) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncAddExclusivePublication> for aeron_async_add_exclusive_publication_t {
    #[inline]
    fn from(value: AeronAsyncAddExclusivePublication) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_add_exclusive_publication_t> for AeronAsyncAddExclusivePublication {
    #[inline]
    fn from(value: *const aeron_async_add_exclusive_publication_t) -> Self {
        AeronAsyncAddExclusivePublication {
            inner: CResource::Borrowed(value as *mut aeron_async_add_exclusive_publication_t),
        }
    }
}
impl From<aeron_async_add_exclusive_publication_t> for AeronAsyncAddExclusivePublication {
    #[inline]
    fn from(value: aeron_async_add_exclusive_publication_t) -> Self {
        AeronAsyncAddExclusivePublication {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl AeronExclusivePublication {
    #[inline]
    pub fn new(async_: &AeronAsyncAddExclusivePublication) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_async_add_exclusive_publication_poll(ctx_field, async_.into())
            },
            None,
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        })
    }
}
impl Aeron {
    #[inline]
    pub fn async_add_exclusive_publication(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
    ) -> Result<AeronAsyncAddExclusivePublication, AeronCError> {
        let mut result = AeronAsyncAddExclusivePublication::new(self, uri, stream_id);
        if let Ok(result) = &mut result {
            result.inner.add_dependency(self.clone());
        }
        result
    }
}
impl Aeron {
    #[inline]
    pub fn add_exclusive_publication(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
        timeout: std::time::Duration,
    ) -> Result<AeronExclusivePublication, AeronCError> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(poller) = AeronAsyncAddExclusivePublication::new(self, uri, stream_id) {
                while start.elapsed() <= timeout {
                    if let Some(result) = poller.poll()? {
                        return Ok(result);
                    }
                    #[cfg(debug_assertions)]
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            if start.elapsed() > timeout {
                log::error!("failed async poll for {:?}", self);
                return Err(AeronErrorType::TimedOut.into());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
impl AeronAsyncAddExclusivePublication {
    #[inline]
    pub fn new(client: &Aeron, uri: &std::ffi::CStr, stream_id: i32) -> Result<Self, AeronCError> {
        let resource_async = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_async_add_exclusive_publication(
                    ctx_field,
                    client.into(),
                    uri.as_ptr(),
                    stream_id.into(),
                )
            },
            None,
            false,
            None,
        )?;
        let result = Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_async)),
        };
        result.inner.add_dependency(client.clone());
        Ok(result)
    }
    pub fn poll(&self) -> Result<Option<AeronExclusivePublication>, AeronCError> {
        let mut result = AeronExclusivePublication::new(self);
        if let Ok(result) = &mut result {
            unsafe {
                for d in (&mut *self.inner.as_owned().unwrap().dependencies.get()).iter_mut() {
                    result.inner.add_dependency(d.clone());
                }
                result.inner.as_owned().unwrap().auto_close.set(true);
            }
        }
        match result {
            Ok(result) => Ok(Some(result)),
            Err(AeronCError { code }) if code == 0 => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_blocking(
        &self,
        timeout: std::time::Duration,
    ) -> Result<AeronExclusivePublication, AeronCError> {
        if let Some(result) = self.poll()? {
            return Ok(result);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Some(result) = self.poll()? {
                return Ok(result);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {:?}", self);
        Err(AeronErrorType::TimedOut.into())
    }
}
#[derive(Clone)]
pub struct AeronAsyncAddPublication {
    inner: CResource<aeron_async_add_publication_t>,
}
impl core::fmt::Debug for AeronAsyncAddPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncAddPublication))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncAddPublication))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncAddPublication {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_async_add_publication_t)
                );
                let inst: aeron_async_add_publication_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_async_add_publication_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_async_add_publication_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Gets the registration id for addition of the publication. Note that using this after a call to poll the succeeds or"]
    #[doc = " errors is undefined behaviour. As the async_add_publication_t may have been freed."]
    #[doc = ""]
    #[doc = " \n# Return\n registration id for the publication."]
    pub fn get_registration_id(&self) -> i64 {
        unsafe {
            let result = aeron_async_add_publication_get_registration_id(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_add_publication_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_add_publication_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_add_publication_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncAddPublication {
    type Target = aeron_async_add_publication_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_add_publication_t> for AeronAsyncAddPublication {
    #[inline]
    fn from(value: *mut aeron_async_add_publication_t) -> Self {
        AeronAsyncAddPublication {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncAddPublication> for *mut aeron_async_add_publication_t {
    #[inline]
    fn from(value: AeronAsyncAddPublication) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncAddPublication> for *mut aeron_async_add_publication_t {
    #[inline]
    fn from(value: &AeronAsyncAddPublication) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncAddPublication> for aeron_async_add_publication_t {
    #[inline]
    fn from(value: AeronAsyncAddPublication) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_add_publication_t> for AeronAsyncAddPublication {
    #[inline]
    fn from(value: *const aeron_async_add_publication_t) -> Self {
        AeronAsyncAddPublication {
            inner: CResource::Borrowed(value as *mut aeron_async_add_publication_t),
        }
    }
}
impl From<aeron_async_add_publication_t> for AeronAsyncAddPublication {
    #[inline]
    fn from(value: aeron_async_add_publication_t) -> Self {
        AeronAsyncAddPublication {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl AeronPublication {
    #[inline]
    pub fn new(async_: &AeronAsyncAddPublication) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_async_add_publication_poll(ctx_field, async_.into()) },
            None,
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        })
    }
}
impl Aeron {
    #[inline]
    pub fn async_add_publication(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
    ) -> Result<AeronAsyncAddPublication, AeronCError> {
        let mut result = AeronAsyncAddPublication::new(self, uri, stream_id);
        if let Ok(result) = &mut result {
            result.inner.add_dependency(self.clone());
        }
        result
    }
}
impl Aeron {
    #[inline]
    pub fn add_publication(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
        timeout: std::time::Duration,
    ) -> Result<AeronPublication, AeronCError> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(poller) = AeronAsyncAddPublication::new(self, uri, stream_id) {
                while start.elapsed() <= timeout {
                    if let Some(result) = poller.poll()? {
                        return Ok(result);
                    }
                    #[cfg(debug_assertions)]
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            if start.elapsed() > timeout {
                log::error!("failed async poll for {:?}", self);
                return Err(AeronErrorType::TimedOut.into());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
impl AeronAsyncAddPublication {
    #[inline]
    pub fn new(client: &Aeron, uri: &std::ffi::CStr, stream_id: i32) -> Result<Self, AeronCError> {
        let resource_async = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_async_add_publication(
                    ctx_field,
                    client.into(),
                    uri.as_ptr(),
                    stream_id.into(),
                )
            },
            None,
            false,
            None,
        )?;
        let result = Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_async)),
        };
        result.inner.add_dependency(client.clone());
        Ok(result)
    }
    pub fn poll(&self) -> Result<Option<AeronPublication>, AeronCError> {
        let mut result = AeronPublication::new(self);
        if let Ok(result) = &mut result {
            unsafe {
                for d in (&mut *self.inner.as_owned().unwrap().dependencies.get()).iter_mut() {
                    result.inner.add_dependency(d.clone());
                }
                result.inner.as_owned().unwrap().auto_close.set(true);
            }
        }
        match result {
            Ok(result) => Ok(Some(result)),
            Err(AeronCError { code }) if code == 0 => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_blocking(
        &self,
        timeout: std::time::Duration,
    ) -> Result<AeronPublication, AeronCError> {
        if let Some(result) = self.poll()? {
            return Ok(result);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Some(result) = self.poll()? {
                return Ok(result);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {:?}", self);
        Err(AeronErrorType::TimedOut.into())
    }
}
#[derive(Clone)]
pub struct AeronAsyncAddSubscription {
    inner: CResource<aeron_async_add_subscription_t>,
}
impl core::fmt::Debug for AeronAsyncAddSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncAddSubscription))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncAddSubscription))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncAddSubscription {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_async_add_subscription_t)
                );
                let inst: aeron_async_add_subscription_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_async_add_subscription_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_async_add_subscription_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Gets the registration id for addition of the subscription. Note that using this after a call to poll the succeeds or"]
    #[doc = " errors is undefined behaviour. As the async_add_subscription_t may have been freed."]
    #[doc = ""]
    #[doc = " \n# Return\n registration id for the subscription."]
    pub fn get_registration_id(&self) -> i64 {
        unsafe {
            let result = aeron_async_add_subscription_get_registration_id(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_add_subscription_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_add_subscription_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_add_subscription_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncAddSubscription {
    type Target = aeron_async_add_subscription_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_add_subscription_t> for AeronAsyncAddSubscription {
    #[inline]
    fn from(value: *mut aeron_async_add_subscription_t) -> Self {
        AeronAsyncAddSubscription {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncAddSubscription> for *mut aeron_async_add_subscription_t {
    #[inline]
    fn from(value: AeronAsyncAddSubscription) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncAddSubscription> for *mut aeron_async_add_subscription_t {
    #[inline]
    fn from(value: &AeronAsyncAddSubscription) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncAddSubscription> for aeron_async_add_subscription_t {
    #[inline]
    fn from(value: AeronAsyncAddSubscription) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_add_subscription_t> for AeronAsyncAddSubscription {
    #[inline]
    fn from(value: *const aeron_async_add_subscription_t) -> Self {
        AeronAsyncAddSubscription {
            inner: CResource::Borrowed(value as *mut aeron_async_add_subscription_t),
        }
    }
}
impl From<aeron_async_add_subscription_t> for AeronAsyncAddSubscription {
    #[inline]
    fn from(value: aeron_async_add_subscription_t) -> Self {
        AeronAsyncAddSubscription {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl AeronSubscription {
    #[inline]
    pub fn new(async_: &AeronAsyncAddSubscription) -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_async_add_subscription_poll(ctx_field, async_.into()) },
            None,
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        })
    }
}
impl Aeron {
    #[inline]
    pub fn async_add_subscription<
        AeronAvailableImageHandlerImpl: AeronAvailableImageCallback,
        AeronUnavailableImageHandlerImpl: AeronUnavailableImageCallback,
    >(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
        on_available_image_handler: Option<&Handler<AeronAvailableImageHandlerImpl>>,
        on_unavailable_image_handler: Option<&Handler<AeronUnavailableImageHandlerImpl>>,
    ) -> Result<AeronAsyncAddSubscription, AeronCError> {
        let mut result = AeronAsyncAddSubscription::new(
            self,
            uri,
            stream_id,
            on_available_image_handler,
            on_unavailable_image_handler,
        );
        if let Ok(result) = &mut result {
            result.inner.add_dependency(self.clone());
        }
        result
    }
}
impl Aeron {
    #[inline]
    pub fn add_subscription<
        AeronAvailableImageHandlerImpl: AeronAvailableImageCallback,
        AeronUnavailableImageHandlerImpl: AeronUnavailableImageCallback,
    >(
        &self,
        uri: &std::ffi::CStr,
        stream_id: i32,
        on_available_image_handler: Option<&Handler<AeronAvailableImageHandlerImpl>>,
        on_unavailable_image_handler: Option<&Handler<AeronUnavailableImageHandlerImpl>>,
        timeout: std::time::Duration,
    ) -> Result<AeronSubscription, AeronCError> {
        let start = std::time::Instant::now();
        loop {
            if let Ok(poller) = AeronAsyncAddSubscription::new(
                self,
                uri,
                stream_id,
                on_available_image_handler,
                on_unavailable_image_handler,
            ) {
                while start.elapsed() <= timeout {
                    if let Some(result) = poller.poll()? {
                        return Ok(result);
                    }
                    #[cfg(debug_assertions)]
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
            if start.elapsed() > timeout {
                log::error!("failed async poll for {:?}", self);
                return Err(AeronErrorType::TimedOut.into());
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
impl AeronAsyncAddSubscription {
    #[inline]
    pub fn new<
        AeronAvailableImageHandlerImpl: AeronAvailableImageCallback,
        AeronUnavailableImageHandlerImpl: AeronUnavailableImageCallback,
    >(
        client: &Aeron,
        uri: &std::ffi::CStr,
        stream_id: i32,
        on_available_image_handler: Option<&Handler<AeronAvailableImageHandlerImpl>>,
        on_unavailable_image_handler: Option<&Handler<AeronUnavailableImageHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let resource_async = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_async_add_subscription(
                    ctx_field,
                    client.into(),
                    uri.as_ptr(),
                    stream_id.into(),
                    {
                        let callback: aeron_on_available_image_t = if on_available_image_handler
                            .is_none()
                        {
                            None
                        } else {
                            Some(
                                aeron_on_available_image_t_callback::<AeronAvailableImageHandlerImpl>,
                            )
                        };
                        callback
                    },
                    on_available_image_handler
                        .map(|m| m.as_raw())
                        .unwrap_or_else(|| std::ptr::null_mut()),
                    {
                        let callback: aeron_on_unavailable_image_t =
                            if on_unavailable_image_handler.is_none() {
                                None
                            } else {
                                Some(
                                    aeron_on_unavailable_image_t_callback::<
                                        AeronUnavailableImageHandlerImpl,
                                    >,
                                )
                            };
                        callback
                    },
                    on_unavailable_image_handler
                        .map(|m| m.as_raw())
                        .unwrap_or_else(|| std::ptr::null_mut()),
                )
            },
            None,
            false,
            None,
        )?;
        let result = Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_async)),
        };
        result.inner.add_dependency(client.clone());
        Ok(result)
    }
    pub fn poll(&self) -> Result<Option<AeronSubscription>, AeronCError> {
        let mut result = AeronSubscription::new(self);
        if let Ok(result) = &mut result {
            unsafe {
                for d in (&mut *self.inner.as_owned().unwrap().dependencies.get()).iter_mut() {
                    result.inner.add_dependency(d.clone());
                }
                result.inner.as_owned().unwrap().auto_close.set(true);
            }
        }
        match result {
            Ok(result) => Ok(Some(result)),
            Err(AeronCError { code }) if code == 0 => Ok(None),
            Err(e) => Err(e),
        }
    }
    pub fn poll_blocking(
        &self,
        timeout: std::time::Duration,
    ) -> Result<AeronSubscription, AeronCError> {
        if let Some(result) = self.poll()? {
            return Ok(result);
        }
        let time = std::time::Instant::now();
        while time.elapsed() < timeout {
            if let Some(result) = self.poll()? {
                return Ok(result);
            }
            #[cfg(debug_assertions)]
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::error!("failed async poll for {:?}", self);
        Err(AeronErrorType::TimedOut.into())
    }
}
#[derive(Clone)]
pub struct AeronAsyncDestinationById {
    inner: CResource<aeron_async_destination_by_id_t>,
}
impl core::fmt::Debug for AeronAsyncDestinationById {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncDestinationById))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncDestinationById))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncDestinationById {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_async_destination_by_id_t)
                );
                let inst: aeron_async_destination_by_id_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_async_destination_by_id_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_async_destination_by_id_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_destination_by_id_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_destination_by_id_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_destination_by_id_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncDestinationById {
    type Target = aeron_async_destination_by_id_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_destination_by_id_t> for AeronAsyncDestinationById {
    #[inline]
    fn from(value: *mut aeron_async_destination_by_id_t) -> Self {
        AeronAsyncDestinationById {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncDestinationById> for *mut aeron_async_destination_by_id_t {
    #[inline]
    fn from(value: AeronAsyncDestinationById) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncDestinationById> for *mut aeron_async_destination_by_id_t {
    #[inline]
    fn from(value: &AeronAsyncDestinationById) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncDestinationById> for aeron_async_destination_by_id_t {
    #[inline]
    fn from(value: AeronAsyncDestinationById) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_destination_by_id_t> for AeronAsyncDestinationById {
    #[inline]
    fn from(value: *const aeron_async_destination_by_id_t) -> Self {
        AeronAsyncDestinationById {
            inner: CResource::Borrowed(value as *mut aeron_async_destination_by_id_t),
        }
    }
}
impl From<aeron_async_destination_by_id_t> for AeronAsyncDestinationById {
    #[inline]
    fn from(value: aeron_async_destination_by_id_t) -> Self {
        AeronAsyncDestinationById {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronAsyncDestination {
    inner: CResource<aeron_async_destination_t>,
}
impl core::fmt::Debug for AeronAsyncDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAsyncDestination))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAsyncDestination))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAsyncDestination {
    #[doc = "Add a destination manually to a multi-destination-cast publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `publication` to add destination to."]
    #[doc = " \n - `uri` for the destination to add."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn aeron_publication_async_add_destination(
        client: &Aeron,
        publication: &AeronPublication,
        uri: &std::ffi::CStr,
    ) -> Result<Self, AeronCError> {
        let client_copy = client.clone();
        let client: *mut aeron_t = client.into();
        let publication_copy = publication.clone();
        let publication: *mut aeron_publication_t = publication.into();
        let uri: *const ::std::os::raw::c_char = uri.as_ptr();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_publication_async_add_destination(ctx_field, client, publication, uri)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_publication_async_remove_destination(
                    ctx_field,
                    client.into(),
                    publication.into(),
                    uri.into(),
                )
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[doc = "Add a destination manually to a multi-destination-cast exclusive publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `publication` to add destination to."]
    #[doc = " \n - `uri` for the destination to add."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn aeron_exclusive_publication_async_add_destination(
        client: &Aeron,
        publication: &AeronExclusivePublication,
        uri: &std::ffi::CStr,
    ) -> Result<Self, AeronCError> {
        let client_copy = client.clone();
        let client: *mut aeron_t = client.into();
        let publication_copy = publication.clone();
        let publication: *mut aeron_exclusive_publication_t = publication.into();
        let uri: *const ::std::os::raw::c_char = uri.as_ptr();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_exclusive_publication_async_add_destination(
                    ctx_field,
                    client,
                    publication,
                    uri,
                )
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_exclusive_publication_async_remove_destination(
                    ctx_field,
                    client.into(),
                    publication.into(),
                    uri.into(),
                )
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[doc = "Add a destination manually to a multi-destination-subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `subscription` to add destination to."]
    #[doc = " \n - `uri` for the destination to add."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn aeron_subscription_async_add_destination(
        client: &Aeron,
        subscription: &AeronSubscription,
        uri: &std::ffi::CStr,
    ) -> Result<Self, AeronCError> {
        let client_copy = client.clone();
        let client: *mut aeron_t = client.into();
        let subscription_copy = subscription.clone();
        let subscription: *mut aeron_subscription_t = subscription.into();
        let uri: *const ::std::os::raw::c_char = uri.as_ptr();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_subscription_async_add_destination(ctx_field, client, subscription, uri)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_subscription_async_remove_destination(
                    ctx_field,
                    client.into(),
                    subscription.into(),
                    uri.into(),
                )
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Poll the completion of the add/remove of a destination to/from a publication."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for not complete (try again), 1 for completed successfully, or -1 for an error."]
    pub fn aeron_publication_async_destination_poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_publication_async_destination_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the completion of the add/remove of a destination to/from an exclusive publication."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for not complete (try again), 1 for completed successfully, or -1 for an error."]
    pub fn aeron_exclusive_publication_async_destination_poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_exclusive_publication_async_destination_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the completion of add/remove of a destination to/from a subscription."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for not complete (try again), 1 for completed successfully, or -1 for an error."]
    pub fn aeron_subscription_async_destination_poll(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_async_destination_poll(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Gets the registration_id for the destination command supplied. Note that this is the correlation_id used for"]
    #[doc = " the specified destination command, not the registration_id for the original parent resource (publication,"]
    #[doc = " subscription)."]
    #[doc = ""]
    #[doc = " \n# Return\n correlation_id sent to driver."]
    pub fn get_registration_id(&self) -> i64 {
        unsafe {
            let result = aeron_async_destination_get_registration_id(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_async_destination_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_async_destination_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_async_destination_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAsyncDestination {
    type Target = aeron_async_destination_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_async_destination_t> for AeronAsyncDestination {
    #[inline]
    fn from(value: *mut aeron_async_destination_t) -> Self {
        AeronAsyncDestination {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAsyncDestination> for *mut aeron_async_destination_t {
    #[inline]
    fn from(value: AeronAsyncDestination) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAsyncDestination> for *mut aeron_async_destination_t {
    #[inline]
    fn from(value: &AeronAsyncDestination) -> Self {
        value.get_inner()
    }
}
impl From<AeronAsyncDestination> for aeron_async_destination_t {
    #[inline]
    fn from(value: AeronAsyncDestination) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_async_destination_t> for AeronAsyncDestination {
    #[inline]
    fn from(value: *const aeron_async_destination_t) -> Self {
        AeronAsyncDestination {
            inner: CResource::Borrowed(value as *mut aeron_async_destination_t),
        }
    }
}
impl From<aeron_async_destination_t> for AeronAsyncDestination {
    #[inline]
    fn from(value: aeron_async_destination_t) -> Self {
        AeronAsyncDestination {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = "Structure used to hold information for a try_claim function call."]
#[derive(Clone)]
pub struct AeronBufferClaim {
    inner: CResource<aeron_buffer_claim_t>,
}
impl core::fmt::Debug for AeronBufferClaim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronBufferClaim))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronBufferClaim))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronBufferClaim {
    #[inline]
    pub fn new(frame_header: *mut u8, data: &mut [u8]) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_buffer_claim_t {
                    frame_header: frame_header.into(),
                    data: data.as_ptr() as *mut _,
                    length: data.len(),
                };
                let inner_ptr: *mut aeron_buffer_claim_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_buffer_claim_t)
                );
                let inst: aeron_buffer_claim_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_buffer_claim_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_buffer_claim_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> *mut u8 {
        self.frame_header.into()
    }
    #[inline]
    pub fn data(&self) -> &mut [u8] {
        unsafe {
            if self.data.is_null() {
                &mut [] as &mut [_]
            } else {
                std::slice::from_raw_parts_mut(self.data, self.length.try_into().unwrap())
            }
        }
    }
    #[inline]
    pub fn length(&self) -> usize {
        self.length.into()
    }
    #[inline]
    #[doc = "Commit the given buffer_claim as a complete message available for consumption."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn commit(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_buffer_claim_commit(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Abort the given buffer_claim and assign its position as padding."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn abort(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_buffer_claim_abort(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_buffer_claim_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_buffer_claim_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_buffer_claim_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronBufferClaim {
    type Target = aeron_buffer_claim_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_buffer_claim_t> for AeronBufferClaim {
    #[inline]
    fn from(value: *mut aeron_buffer_claim_t) -> Self {
        AeronBufferClaim {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronBufferClaim> for *mut aeron_buffer_claim_t {
    #[inline]
    fn from(value: AeronBufferClaim) -> Self {
        value.get_inner()
    }
}
impl From<&AeronBufferClaim> for *mut aeron_buffer_claim_t {
    #[inline]
    fn from(value: &AeronBufferClaim) -> Self {
        value.get_inner()
    }
}
impl From<AeronBufferClaim> for aeron_buffer_claim_t {
    #[inline]
    fn from(value: AeronBufferClaim) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_buffer_claim_t> for AeronBufferClaim {
    #[inline]
    fn from(value: *const aeron_buffer_claim_t) -> Self {
        AeronBufferClaim {
            inner: CResource::Borrowed(value as *mut aeron_buffer_claim_t),
        }
    }
}
impl From<aeron_buffer_claim_t> for AeronBufferClaim {
    #[inline]
    fn from(value: aeron_buffer_claim_t) -> Self {
        AeronBufferClaim {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronBufferClaim {
    fn default() -> Self {
        AeronBufferClaim::new_zeroed_on_heap()
    }
}
impl AeronBufferClaim {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronClientRegisteringResource {
    inner: CResource<aeron_client_registering_resource_t>,
}
impl core::fmt::Debug for AeronClientRegisteringResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronClientRegisteringResource))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronClientRegisteringResource))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronClientRegisteringResource {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_client_registering_resource_t)
                );
                let inst: aeron_client_registering_resource_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_client_registering_resource_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_client_registering_resource_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_client_registering_resource_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_client_registering_resource_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_client_registering_resource_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronClientRegisteringResource {
    type Target = aeron_client_registering_resource_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_client_registering_resource_t> for AeronClientRegisteringResource {
    #[inline]
    fn from(value: *mut aeron_client_registering_resource_t) -> Self {
        AeronClientRegisteringResource {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronClientRegisteringResource> for *mut aeron_client_registering_resource_t {
    #[inline]
    fn from(value: AeronClientRegisteringResource) -> Self {
        value.get_inner()
    }
}
impl From<&AeronClientRegisteringResource> for *mut aeron_client_registering_resource_t {
    #[inline]
    fn from(value: &AeronClientRegisteringResource) -> Self {
        value.get_inner()
    }
}
impl From<AeronClientRegisteringResource> for aeron_client_registering_resource_t {
    #[inline]
    fn from(value: AeronClientRegisteringResource) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_client_registering_resource_t> for AeronClientRegisteringResource {
    #[inline]
    fn from(value: *const aeron_client_registering_resource_t) -> Self {
        AeronClientRegisteringResource {
            inner: CResource::Borrowed(value as *mut aeron_client_registering_resource_t),
        }
    }
}
impl From<aeron_client_registering_resource_t> for AeronClientRegisteringResource {
    #[inline]
    fn from(value: aeron_client_registering_resource_t) -> Self {
        AeronClientRegisteringResource {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronCncConstants {
    inner: CResource<aeron_cnc_constants_t>,
}
impl core::fmt::Debug for AeronCncConstants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCncConstants))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCncConstants))
                .field("inner", &self.inner)
                .field(stringify!(cnc_version), &self.cnc_version())
                .field(
                    stringify!(to_driver_buffer_length),
                    &self.to_driver_buffer_length(),
                )
                .field(
                    stringify!(to_clients_buffer_length),
                    &self.to_clients_buffer_length(),
                )
                .field(
                    stringify!(counter_metadata_buffer_length),
                    &self.counter_metadata_buffer_length(),
                )
                .field(
                    stringify!(counter_values_buffer_length),
                    &self.counter_values_buffer_length(),
                )
                .field(
                    stringify!(error_log_buffer_length),
                    &self.error_log_buffer_length(),
                )
                .field(
                    stringify!(client_liveness_timeout),
                    &self.client_liveness_timeout(),
                )
                .field(stringify!(start_timestamp), &self.start_timestamp())
                .field(stringify!(pid), &self.pid())
                .field(stringify!(file_page_size), &self.file_page_size())
                .finish()
        }
    }
}
impl AeronCncConstants {
    #[inline]
    pub fn new(
        cnc_version: i32,
        to_driver_buffer_length: i32,
        to_clients_buffer_length: i32,
        counter_metadata_buffer_length: i32,
        counter_values_buffer_length: i32,
        error_log_buffer_length: i32,
        client_liveness_timeout: i64,
        start_timestamp: i64,
        pid: i64,
        file_page_size: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_cnc_constants_t {
                    cnc_version: cnc_version.into(),
                    to_driver_buffer_length: to_driver_buffer_length.into(),
                    to_clients_buffer_length: to_clients_buffer_length.into(),
                    counter_metadata_buffer_length: counter_metadata_buffer_length.into(),
                    counter_values_buffer_length: counter_values_buffer_length.into(),
                    error_log_buffer_length: error_log_buffer_length.into(),
                    client_liveness_timeout: client_liveness_timeout.into(),
                    start_timestamp: start_timestamp.into(),
                    pid: pid.into(),
                    file_page_size: file_page_size.into(),
                };
                let inner_ptr: *mut aeron_cnc_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_cnc_constants_t)
                );
                let inst: aeron_cnc_constants_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_cnc_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_cnc_constants_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn cnc_version(&self) -> i32 {
        self.cnc_version.into()
    }
    #[inline]
    pub fn to_driver_buffer_length(&self) -> i32 {
        self.to_driver_buffer_length.into()
    }
    #[inline]
    pub fn to_clients_buffer_length(&self) -> i32 {
        self.to_clients_buffer_length.into()
    }
    #[inline]
    pub fn counter_metadata_buffer_length(&self) -> i32 {
        self.counter_metadata_buffer_length.into()
    }
    #[inline]
    pub fn counter_values_buffer_length(&self) -> i32 {
        self.counter_values_buffer_length.into()
    }
    #[inline]
    pub fn error_log_buffer_length(&self) -> i32 {
        self.error_log_buffer_length.into()
    }
    #[inline]
    pub fn client_liveness_timeout(&self) -> i64 {
        self.client_liveness_timeout.into()
    }
    #[inline]
    pub fn start_timestamp(&self) -> i64 {
        self.start_timestamp.into()
    }
    #[inline]
    pub fn pid(&self) -> i64 {
        self.pid.into()
    }
    #[inline]
    pub fn file_page_size(&self) -> i32 {
        self.file_page_size.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_cnc_constants_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_cnc_constants_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_cnc_constants_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCncConstants {
    type Target = aeron_cnc_constants_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_cnc_constants_t> for AeronCncConstants {
    #[inline]
    fn from(value: *mut aeron_cnc_constants_t) -> Self {
        AeronCncConstants {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCncConstants> for *mut aeron_cnc_constants_t {
    #[inline]
    fn from(value: AeronCncConstants) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCncConstants> for *mut aeron_cnc_constants_t {
    #[inline]
    fn from(value: &AeronCncConstants) -> Self {
        value.get_inner()
    }
}
impl From<AeronCncConstants> for aeron_cnc_constants_t {
    #[inline]
    fn from(value: AeronCncConstants) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_cnc_constants_t> for AeronCncConstants {
    #[inline]
    fn from(value: *const aeron_cnc_constants_t) -> Self {
        AeronCncConstants {
            inner: CResource::Borrowed(value as *mut aeron_cnc_constants_t),
        }
    }
}
impl From<aeron_cnc_constants_t> for AeronCncConstants {
    #[inline]
    fn from(value: aeron_cnc_constants_t) -> Self {
        AeronCncConstants {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCncConstants {
    fn default() -> Self {
        AeronCncConstants::new_zeroed_on_heap()
    }
}
impl AeronCncConstants {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCncMetadata {
    inner: CResource<aeron_cnc_metadata_t>,
}
impl core::fmt::Debug for AeronCncMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCncMetadata))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCncMetadata))
                .field("inner", &self.inner)
                .field(stringify!(cnc_version), &self.cnc_version())
                .field(
                    stringify!(to_driver_buffer_length),
                    &self.to_driver_buffer_length(),
                )
                .field(
                    stringify!(to_clients_buffer_length),
                    &self.to_clients_buffer_length(),
                )
                .field(
                    stringify!(counter_metadata_buffer_length),
                    &self.counter_metadata_buffer_length(),
                )
                .field(
                    stringify!(counter_values_buffer_length),
                    &self.counter_values_buffer_length(),
                )
                .field(
                    stringify!(error_log_buffer_length),
                    &self.error_log_buffer_length(),
                )
                .field(
                    stringify!(client_liveness_timeout),
                    &self.client_liveness_timeout(),
                )
                .field(stringify!(start_timestamp), &self.start_timestamp())
                .field(stringify!(pid), &self.pid())
                .field(stringify!(file_page_size), &self.file_page_size())
                .finish()
        }
    }
}
impl AeronCncMetadata {
    #[inline]
    pub fn new(
        cnc_version: i32,
        to_driver_buffer_length: i32,
        to_clients_buffer_length: i32,
        counter_metadata_buffer_length: i32,
        counter_values_buffer_length: i32,
        error_log_buffer_length: i32,
        client_liveness_timeout: i64,
        start_timestamp: i64,
        pid: i64,
        file_page_size: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_cnc_metadata_t {
                    cnc_version: cnc_version.into(),
                    to_driver_buffer_length: to_driver_buffer_length.into(),
                    to_clients_buffer_length: to_clients_buffer_length.into(),
                    counter_metadata_buffer_length: counter_metadata_buffer_length.into(),
                    counter_values_buffer_length: counter_values_buffer_length.into(),
                    error_log_buffer_length: error_log_buffer_length.into(),
                    client_liveness_timeout: client_liveness_timeout.into(),
                    start_timestamp: start_timestamp.into(),
                    pid: pid.into(),
                    file_page_size: file_page_size.into(),
                };
                let inner_ptr: *mut aeron_cnc_metadata_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_cnc_metadata_t)
                );
                let inst: aeron_cnc_metadata_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_cnc_metadata_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_cnc_metadata_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn cnc_version(&self) -> i32 {
        self.cnc_version.into()
    }
    #[inline]
    pub fn to_driver_buffer_length(&self) -> i32 {
        self.to_driver_buffer_length.into()
    }
    #[inline]
    pub fn to_clients_buffer_length(&self) -> i32 {
        self.to_clients_buffer_length.into()
    }
    #[inline]
    pub fn counter_metadata_buffer_length(&self) -> i32 {
        self.counter_metadata_buffer_length.into()
    }
    #[inline]
    pub fn counter_values_buffer_length(&self) -> i32 {
        self.counter_values_buffer_length.into()
    }
    #[inline]
    pub fn error_log_buffer_length(&self) -> i32 {
        self.error_log_buffer_length.into()
    }
    #[inline]
    pub fn client_liveness_timeout(&self) -> i64 {
        self.client_liveness_timeout.into()
    }
    #[inline]
    pub fn start_timestamp(&self) -> i64 {
        self.start_timestamp.into()
    }
    #[inline]
    pub fn pid(&self) -> i64 {
        self.pid.into()
    }
    #[inline]
    pub fn file_page_size(&self) -> i32 {
        self.file_page_size.into()
    }
    #[inline]
    pub fn aeron_cnc_version_volatile(&self) -> i32 {
        unsafe {
            let result = aeron_cnc_version_volatile(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_cnc_metadata_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_cnc_metadata_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_cnc_metadata_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCncMetadata {
    type Target = aeron_cnc_metadata_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_cnc_metadata_t> for AeronCncMetadata {
    #[inline]
    fn from(value: *mut aeron_cnc_metadata_t) -> Self {
        AeronCncMetadata {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCncMetadata> for *mut aeron_cnc_metadata_t {
    #[inline]
    fn from(value: AeronCncMetadata) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCncMetadata> for *mut aeron_cnc_metadata_t {
    #[inline]
    fn from(value: &AeronCncMetadata) -> Self {
        value.get_inner()
    }
}
impl From<AeronCncMetadata> for aeron_cnc_metadata_t {
    #[inline]
    fn from(value: AeronCncMetadata) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_cnc_metadata_t> for AeronCncMetadata {
    #[inline]
    fn from(value: *const aeron_cnc_metadata_t) -> Self {
        AeronCncMetadata {
            inner: CResource::Borrowed(value as *mut aeron_cnc_metadata_t),
        }
    }
}
impl From<aeron_cnc_metadata_t> for AeronCncMetadata {
    #[inline]
    fn from(value: aeron_cnc_metadata_t) -> Self {
        AeronCncMetadata {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCncMetadata {
    fn default() -> Self {
        AeronCncMetadata::new_zeroed_on_heap()
    }
}
impl AeronCncMetadata {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCnc {
    inner: CResource<aeron_cnc_t>,
}
impl core::fmt::Debug for AeronCnc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCnc))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCnc))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronCnc {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_cnc_t)
                );
                let inst: aeron_cnc_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_cnc_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_cnc_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Fetch the sets of constant values associated with this command and control file."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `constants` user supplied structure to hold return values."]
    #[doc = " \n# Return\n 0 on success, -1 on failure."]
    pub fn constants(&self, constants: &AeronCncConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_cnc_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fetch the sets of constant values associated with this command and control file."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronCncConstants, AeronCError> {
        let result = AeronCncConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the current file name of the cnc file."]
    #[doc = ""]
    #[doc = " \n# Return\n name of the cnc file"]
    pub fn filename(&self) -> &str {
        unsafe {
            let result = aeron_cnc_filename(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Gets the timestamp of the last heartbeat sent to the media driver from any client."]
    #[doc = ""]
    #[doc = " \n# Return\n last heartbeat timestamp in ms."]
    pub fn to_driver_heartbeat(&self) -> i64 {
        unsafe {
            let result = aeron_cnc_to_driver_heartbeat(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Reads the current error log for this driver."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `callback` called for every distinct error observation"]
    #[doc = " \n - `clientd` client data to be passed to the callback"]
    #[doc = " \n - `since_timestamp` only return errors after this timestamp (0 returns all)"]
    #[doc = " \n# Return\n the number of distinct errors seen"]
    pub fn error_log_read<AeronErrorLogReaderFuncHandlerImpl: AeronErrorLogReaderFuncCallback>(
        &self,
        callback: Option<&Handler<AeronErrorLogReaderFuncHandlerImpl>>,
        since_timestamp: i64,
    ) -> usize {
        unsafe {
            let result = aeron_cnc_error_log_read(
                self.get_inner(),
                {
                    let callback: aeron_error_log_reader_func_t = if callback.is_none() {
                        None
                    } else {
                        Some(
                            aeron_error_log_reader_func_t_callback::<
                                AeronErrorLogReaderFuncHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                callback
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                since_timestamp.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Reads the current error log for this driver."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `callback` called for every distinct error observation"]
    #[doc = " \n - `clientd` client data to be passed to the callback"]
    #[doc = " \n - `since_timestamp` only return errors after this timestamp (0 returns all)"]
    #[doc = " \n# Return\n the number of distinct errors seen"]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn error_log_read_once<
        AeronErrorLogReaderFuncHandlerImpl: FnMut(i32, i64, i64, &str) -> (),
    >(
        &self,
        mut callback: AeronErrorLogReaderFuncHandlerImpl,
        since_timestamp: i64,
    ) -> usize {
        unsafe {
            let result = aeron_cnc_error_log_read(
                self.get_inner(),
                Some(
                    aeron_error_log_reader_func_t_callback_for_once_closure::<
                        AeronErrorLogReaderFuncHandlerImpl,
                    >,
                ),
                &mut callback as *mut _ as *mut std::os::raw::c_void,
                since_timestamp.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Gets a counters reader for this command and control file. This does not need to be closed manually, resources"]
    #[doc = " are tied to the instance of aeron_cnc."]
    #[doc = ""]
    #[doc = " \n# Return\n pointer to a counters reader."]
    pub fn counters_reader(&self) -> AeronCountersReader {
        unsafe {
            let result = aeron_cnc_counters_reader(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Read all of the data loss observations from the report in the same media driver instances as the cnc file."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `entry_func` callback for each observation found"]
    #[doc = " \n# Return\n -1 on failure, number of observations on success (could be 0)."]
    pub fn loss_reporter_read<
        AeronLossReporterReadEntryFuncHandlerImpl: AeronLossReporterReadEntryFuncCallback,
    >(
        &self,
        entry_func: Option<&Handler<AeronLossReporterReadEntryFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_cnc_loss_reporter_read(
                self.get_inner(),
                {
                    let callback: aeron_loss_reporter_read_entry_func_t = if entry_func.is_none() {
                        None
                    } else {
                        Some(
                            aeron_loss_reporter_read_entry_func_t_callback::<
                                AeronLossReporterReadEntryFuncHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                entry_func
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Read all of the data loss observations from the report in the same media driver instances as the cnc file."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `entry_func` callback for each observation found"]
    #[doc = " \n# Return\n -1 on failure, number of observations on success (could be 0)."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn loss_reporter_read_once<
        AeronLossReporterReadEntryFuncHandlerImpl: FnMut(i64, i64, i64, i64, i32, i32, &str, &str) -> (),
    >(
        &self,
        mut entry_func: AeronLossReporterReadEntryFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_cnc_loss_reporter_read(
                self.get_inner(),
                Some(
                    aeron_loss_reporter_read_entry_func_t_callback_for_once_closure::<
                        AeronLossReporterReadEntryFuncHandlerImpl,
                    >,
                ),
                &mut entry_func as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Closes the instance of the aeron cnc and frees its resources."]
    #[doc = ""]
    pub fn close(&self) -> () {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_cnc_close(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn resolve_filename(
        directory: &std::ffi::CStr,
        filename_buffer: *mut ::std::os::raw::c_char,
        filename_buffer_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_cnc_resolve_filename(
                directory.as_ptr(),
                filename_buffer.into(),
                filename_buffer_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_cnc_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_cnc_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_cnc_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCnc {
    type Target = aeron_cnc_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_cnc_t> for AeronCnc {
    #[inline]
    fn from(value: *mut aeron_cnc_t) -> Self {
        AeronCnc {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCnc> for *mut aeron_cnc_t {
    #[inline]
    fn from(value: AeronCnc) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCnc> for *mut aeron_cnc_t {
    #[inline]
    fn from(value: &AeronCnc) -> Self {
        value.get_inner()
    }
}
impl From<AeronCnc> for aeron_cnc_t {
    #[inline]
    fn from(value: AeronCnc) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_cnc_t> for AeronCnc {
    #[inline]
    fn from(value: *const aeron_cnc_t) -> Self {
        AeronCnc {
            inner: CResource::Borrowed(value as *mut aeron_cnc_t),
        }
    }
}
impl From<aeron_cnc_t> for AeronCnc {
    #[inline]
    fn from(value: aeron_cnc_t) -> Self {
        AeronCnc {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronContext {
    inner: CResource<aeron_context_t>,
}
impl core::fmt::Debug for AeronContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronContext))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronContext))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronContext {
    #[doc = "Create a `AeronContext` struct and initialize with default values."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new() -> Result<Self, AeronCError> {
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_context_init(ctx_field) },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_context_close(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    pub fn set_dir(&self, value: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_dir(self.get_inner(), value.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_dir(&self) -> &str {
        unsafe {
            let result = aeron_context_get_dir(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    pub fn set_driver_timeout_ms(&self, value: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_driver_timeout_ms(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_driver_timeout_ms(&self) -> u64 {
        unsafe {
            let result = aeron_context_get_driver_timeout_ms(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_keepalive_interval_ns(&self, value: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_keepalive_interval_ns(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_keepalive_interval_ns(&self) -> u64 {
        unsafe {
            let result = aeron_context_get_keepalive_interval_ns(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_resource_linger_duration_ns(&self, value: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_context_set_resource_linger_duration_ns(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_resource_linger_duration_ns(&self) -> u64 {
        unsafe {
            let result = aeron_context_get_resource_linger_duration_ns(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_idle_sleep_duration_ns(&self) -> u64 {
        unsafe {
            let result = aeron_context_get_idle_sleep_duration_ns(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_idle_sleep_duration_ns(&self, value: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_idle_sleep_duration_ns(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn set_pre_touch_mapped_memory(&self, value: bool) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_pre_touch_mapped_memory(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_pre_touch_mapped_memory(&self) -> bool {
        unsafe {
            let result = aeron_context_get_pre_touch_mapped_memory(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_client_name(&self, value: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_client_name(self.get_inner(), value.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_client_name(&self) -> &str {
        unsafe {
            let result = aeron_context_get_client_name(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    pub fn set_error_handler<AeronErrorHandlerHandlerImpl: AeronErrorHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronErrorHandlerHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_error_handler(
                self.get_inner(),
                {
                    let callback: aeron_error_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_error_handler_t_callback::<AeronErrorHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_error_handler_once<
        AeronErrorHandlerHandlerImpl: FnMut(::std::os::raw::c_int, &str) -> (),
    >(
        &self,
        mut handler: AeronErrorHandlerHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_error_handler(
                self.get_inner(),
                Some(
                    aeron_error_handler_t_callback_for_once_closure::<AeronErrorHandlerHandlerImpl>,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_error_handler(&self) -> aeron_error_handler_t {
        unsafe {
            let result = aeron_context_get_error_handler(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_error_handler_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_error_handler_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_publication_error_frame_handler<
        AeronPublicationErrorFrameHandlerHandlerImpl: AeronPublicationErrorFrameHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronPublicationErrorFrameHandlerHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_publication_error_frame_handler(
                self.get_inner(),
                {
                    let callback: aeron_publication_error_frame_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_publication_error_frame_handler_t_callback::<
                                AeronPublicationErrorFrameHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_publication_error_frame_handler_once<
        AeronPublicationErrorFrameHandlerHandlerImpl: FnMut(AeronPublicationErrorValues) -> (),
    >(
        &self,
        mut handler: AeronPublicationErrorFrameHandlerHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_publication_error_frame_handler(
                self.get_inner(),
                Some(
                    aeron_publication_error_frame_handler_t_callback_for_once_closure::<
                        AeronPublicationErrorFrameHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_publication_error_frame_handler(&self) -> aeron_publication_error_frame_handler_t {
        unsafe {
            let result = aeron_context_get_publication_error_frame_handler(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_publication_error_frame_handler_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result =
                aeron_context_get_publication_error_frame_handler_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_new_publication<AeronNewPublicationHandlerImpl: AeronNewPublicationCallback>(
        &self,
        handler: Option<&Handler<AeronNewPublicationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_publication(
                self.get_inner(),
                {
                    let callback: aeron_on_new_publication_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_on_new_publication_t_callback::<AeronNewPublicationHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_new_publication_once<
        AeronNewPublicationHandlerImpl: FnMut(AeronAsyncAddPublication, &str, i32, i32, i64) -> (),
    >(
        &self,
        mut handler: AeronNewPublicationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_publication(
                self.get_inner(),
                Some(
                    aeron_on_new_publication_t_callback_for_once_closure::<
                        AeronNewPublicationHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_new_publication(&self) -> aeron_on_new_publication_t {
        unsafe {
            let result = aeron_context_get_on_new_publication(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_new_publication_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_new_publication_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_new_exclusive_publication<
        AeronNewPublicationHandlerImpl: AeronNewPublicationCallback,
    >(
        &self,
        handler: Option<&Handler<AeronNewPublicationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_exclusive_publication(
                self.get_inner(),
                {
                    let callback: aeron_on_new_publication_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_on_new_publication_t_callback::<AeronNewPublicationHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_new_exclusive_publication_once<
        AeronNewPublicationHandlerImpl: FnMut(AeronAsyncAddPublication, &str, i32, i32, i64) -> (),
    >(
        &self,
        mut handler: AeronNewPublicationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_exclusive_publication(
                self.get_inner(),
                Some(
                    aeron_on_new_publication_t_callback_for_once_closure::<
                        AeronNewPublicationHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_new_exclusive_publication(&self) -> aeron_on_new_publication_t {
        unsafe {
            let result = aeron_context_get_on_new_exclusive_publication(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_new_exclusive_publication_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_new_exclusive_publication_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_new_subscription<
        AeronNewSubscriptionHandlerImpl: AeronNewSubscriptionCallback,
    >(
        &self,
        handler: Option<&Handler<AeronNewSubscriptionHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_subscription(
                self.get_inner(),
                {
                    let callback: aeron_on_new_subscription_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_on_new_subscription_t_callback::<AeronNewSubscriptionHandlerImpl>,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_new_subscription_once<
        AeronNewSubscriptionHandlerImpl: FnMut(AeronAsyncAddSubscription, &str, i32, i64) -> (),
    >(
        &self,
        mut handler: AeronNewSubscriptionHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_new_subscription(
                self.get_inner(),
                Some(
                    aeron_on_new_subscription_t_callback_for_once_closure::<
                        AeronNewSubscriptionHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_new_subscription(&self) -> aeron_on_new_subscription_t {
        unsafe {
            let result = aeron_context_get_on_new_subscription(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_new_subscription_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_new_subscription_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_available_counter<
        AeronAvailableCounterHandlerImpl: AeronAvailableCounterCallback,
    >(
        &self,
        handler: Option<&Handler<AeronAvailableCounterHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_available_counter(
                self.get_inner(),
                {
                    let callback: aeron_on_available_counter_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_on_available_counter_t_callback::<AeronAvailableCounterHandlerImpl>,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_available_counter_once<
        AeronAvailableCounterHandlerImpl: FnMut(AeronCountersReader, i64, i32) -> (),
    >(
        &self,
        mut handler: AeronAvailableCounterHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_available_counter(
                self.get_inner(),
                Some(
                    aeron_on_available_counter_t_callback_for_once_closure::<
                        AeronAvailableCounterHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_available_counter(&self) -> aeron_on_available_counter_t {
        unsafe {
            let result = aeron_context_get_on_available_counter(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_available_counter_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_available_counter_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_unavailable_counter<
        AeronUnavailableCounterHandlerImpl: AeronUnavailableCounterCallback,
    >(
        &self,
        handler: Option<&Handler<AeronUnavailableCounterHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_unavailable_counter(
                self.get_inner(),
                {
                    let callback: aeron_on_unavailable_counter_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_on_unavailable_counter_t_callback::<
                                AeronUnavailableCounterHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_unavailable_counter_once<
        AeronUnavailableCounterHandlerImpl: FnMut(AeronCountersReader, i64, i32) -> (),
    >(
        &self,
        mut handler: AeronUnavailableCounterHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_unavailable_counter(
                self.get_inner(),
                Some(
                    aeron_on_unavailable_counter_t_callback_for_once_closure::<
                        AeronUnavailableCounterHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_unavailable_counter(&self) -> aeron_on_unavailable_counter_t {
        unsafe {
            let result = aeron_context_get_on_unavailable_counter(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_unavailable_counter_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_unavailable_counter_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_on_close_client<AeronCloseClientHandlerImpl: AeronCloseClientCallback>(
        &self,
        handler: Option<&Handler<AeronCloseClientHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_close_client(
                self.get_inner(),
                {
                    let callback: aeron_on_close_client_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_on_close_client_t_callback::<AeronCloseClientHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_on_close_client_once<AeronCloseClientHandlerImpl: FnMut() -> ()>(
        &self,
        mut handler: AeronCloseClientHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_on_close_client(
                self.get_inner(),
                Some(
                    aeron_on_close_client_t_callback_for_once_closure::<AeronCloseClientHandlerImpl>,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_on_close_client(&self) -> aeron_on_close_client_t {
        unsafe {
            let result = aeron_context_get_on_close_client(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_on_close_client_clientd(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_on_close_client_clientd(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Whether to use an invoker to control the conductor agent or spawn a thread."]
    pub fn set_use_conductor_agent_invoker(&self, value: bool) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_context_set_use_conductor_agent_invoker(self.get_inner(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_use_conductor_agent_invoker(&self) -> bool {
        unsafe {
            let result = aeron_context_get_use_conductor_agent_invoker(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn set_agent_on_start_function<
        AeronAgentStartFuncHandlerImpl: AeronAgentStartFuncCallback,
    >(
        &self,
        value: Option<&Handler<AeronAgentStartFuncHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_agent_on_start_function(
                self.get_inner(),
                {
                    let callback: aeron_agent_on_start_func_t = if value.is_none() {
                        None
                    } else {
                        Some(aeron_agent_on_start_func_t_callback::<AeronAgentStartFuncHandlerImpl>)
                    };
                    callback
                },
                value
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn set_agent_on_start_function_once<AeronAgentStartFuncHandlerImpl: FnMut(&str) -> ()>(
        &self,
        mut value: AeronAgentStartFuncHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_set_agent_on_start_function(
                self.get_inner(),
                Some(
                    aeron_agent_on_start_func_t_callback_for_once_closure::<
                        AeronAgentStartFuncHandlerImpl,
                    >,
                ),
                &mut value as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get_agent_on_start_function(&self) -> aeron_agent_on_start_func_t {
        unsafe {
            let result = aeron_context_get_agent_on_start_function(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn get_agent_on_start_state(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_context_get_agent_on_start_state(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Close and delete `AeronContext` struct."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_context_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Request the media driver terminates operation and closes all resources."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `directory`    in which the media driver is running."]
    #[doc = " \n - `token_buffer` containing the authentication token confirming the client is allowed to terminate the driver."]
    #[doc = " \n - `token_length` of the token in the buffer."]
    #[doc = " \n# Return\n"]
    pub fn request_driver_termination(
        directory: &std::ffi::CStr,
        token_buffer: *const u8,
        token_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_context_request_driver_termination(
                directory.as_ptr(),
                token_buffer.into(),
                token_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_context_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_context_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_context_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronContext {
    type Target = aeron_context_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_context_t> for AeronContext {
    #[inline]
    fn from(value: *mut aeron_context_t) -> Self {
        AeronContext {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronContext> for *mut aeron_context_t {
    #[inline]
    fn from(value: AeronContext) -> Self {
        value.get_inner()
    }
}
impl From<&AeronContext> for *mut aeron_context_t {
    #[inline]
    fn from(value: &AeronContext) -> Self {
        value.get_inner()
    }
}
impl From<AeronContext> for aeron_context_t {
    #[inline]
    fn from(value: AeronContext) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_context_t> for AeronContext {
    #[inline]
    fn from(value: *const aeron_context_t) -> Self {
        AeronContext {
            inner: CResource::Borrowed(value as *mut aeron_context_t),
        }
    }
}
impl From<aeron_context_t> for AeronContext {
    #[inline]
    fn from(value: aeron_context_t) -> Self {
        AeronContext {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronControlledFragmentAssembler {
    inner: CResource<aeron_controlled_fragment_assembler_t>,
}
impl core::fmt::Debug for AeronControlledFragmentAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronControlledFragmentAssembler))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronControlledFragmentAssembler))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronControlledFragmentAssembler {
    #[doc = "Create a controlled fragment assembler for use with a subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `delegate` to call on completed"]
    #[doc = " \n - `delegate_clientd` to pass to delegate handler."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        delegate: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let (delegate, delegate_clientd) = (
            {
                let callback: aeron_controlled_fragment_handler_t = if delegate.is_none() {
                    None
                } else {
                    Some(
                        aeron_controlled_fragment_handler_t_callback::<
                            AeronControlledFragmentHandlerHandlerImpl,
                        >,
                    )
                };
                callback
            },
            delegate
                .map(|m| m.as_raw())
                .unwrap_or_else(|| std::ptr::null_mut()),
        );
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_controlled_fragment_assembler_create(ctx_field, delegate, delegate_clientd)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_controlled_fragment_assembler_delete(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Delete a controlled fragment assembler."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn delete(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_controlled_fragment_assembler_delete(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Handler function to be passed for handling fragment assembly."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `clientd` passed in the poll call (must be a `AeronControlledFragmentAssembler`)"]
    #[doc = " \n - `buffer` containing the data."]
    #[doc = " \n - `header` representing the meta data for the data."]
    #[doc = " \n# Return\n The action to be taken with regard to the stream position after the callback."]
    pub fn handler(
        clientd: *mut ::std::os::raw::c_void,
        buffer: &[u8],
        header: &AeronHeader,
    ) -> aeron_controlled_fragment_handler_action_t {
        unsafe {
            let result = aeron_controlled_fragment_assembler_handler(
                clientd.into(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                header.get_inner(),
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_controlled_fragment_assembler_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_controlled_fragment_assembler_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_controlled_fragment_assembler_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronControlledFragmentAssembler {
    type Target = aeron_controlled_fragment_assembler_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_controlled_fragment_assembler_t> for AeronControlledFragmentAssembler {
    #[inline]
    fn from(value: *mut aeron_controlled_fragment_assembler_t) -> Self {
        AeronControlledFragmentAssembler {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronControlledFragmentAssembler> for *mut aeron_controlled_fragment_assembler_t {
    #[inline]
    fn from(value: AeronControlledFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<&AeronControlledFragmentAssembler> for *mut aeron_controlled_fragment_assembler_t {
    #[inline]
    fn from(value: &AeronControlledFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<AeronControlledFragmentAssembler> for aeron_controlled_fragment_assembler_t {
    #[inline]
    fn from(value: AeronControlledFragmentAssembler) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_controlled_fragment_assembler_t> for AeronControlledFragmentAssembler {
    #[inline]
    fn from(value: *const aeron_controlled_fragment_assembler_t) -> Self {
        AeronControlledFragmentAssembler {
            inner: CResource::Borrowed(value as *mut aeron_controlled_fragment_assembler_t),
        }
    }
}
impl From<aeron_controlled_fragment_assembler_t> for AeronControlledFragmentAssembler {
    #[inline]
    fn from(value: aeron_controlled_fragment_assembler_t) -> Self {
        AeronControlledFragmentAssembler {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = "Configuration for a counter that does not change during it's lifetime."]
#[derive(Clone)]
pub struct AeronCounterConstants {
    inner: CResource<aeron_counter_constants_t>,
}
impl core::fmt::Debug for AeronCounterConstants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCounterConstants))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCounterConstants))
                .field("inner", &self.inner)
                .field(stringify!(registration_id), &self.registration_id())
                .field(stringify!(counter_id), &self.counter_id())
                .finish()
        }
    }
}
impl AeronCounterConstants {
    #[inline]
    pub fn new(registration_id: i64, counter_id: i32) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_counter_constants_t {
                    registration_id: registration_id.into(),
                    counter_id: counter_id.into(),
                };
                let inner_ptr: *mut aeron_counter_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counter_constants_t)
                );
                let inst: aeron_counter_constants_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counter_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counter_constants_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn registration_id(&self) -> i64 {
        self.registration_id.into()
    }
    #[inline]
    pub fn counter_id(&self) -> i32 {
        self.counter_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counter_constants_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counter_constants_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counter_constants_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCounterConstants {
    type Target = aeron_counter_constants_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counter_constants_t> for AeronCounterConstants {
    #[inline]
    fn from(value: *mut aeron_counter_constants_t) -> Self {
        AeronCounterConstants {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCounterConstants> for *mut aeron_counter_constants_t {
    #[inline]
    fn from(value: AeronCounterConstants) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCounterConstants> for *mut aeron_counter_constants_t {
    #[inline]
    fn from(value: &AeronCounterConstants) -> Self {
        value.get_inner()
    }
}
impl From<AeronCounterConstants> for aeron_counter_constants_t {
    #[inline]
    fn from(value: AeronCounterConstants) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counter_constants_t> for AeronCounterConstants {
    #[inline]
    fn from(value: *const aeron_counter_constants_t) -> Self {
        AeronCounterConstants {
            inner: CResource::Borrowed(value as *mut aeron_counter_constants_t),
        }
    }
}
impl From<aeron_counter_constants_t> for AeronCounterConstants {
    #[inline]
    fn from(value: aeron_counter_constants_t) -> Self {
        AeronCounterConstants {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCounterConstants {
    fn default() -> Self {
        AeronCounterConstants::new_zeroed_on_heap()
    }
}
impl AeronCounterConstants {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCounterMetadataDescriptor {
    inner: CResource<aeron_counter_metadata_descriptor_t>,
}
impl core::fmt::Debug for AeronCounterMetadataDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCounterMetadataDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCounterMetadataDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(state), &self.state())
                .field(stringify!(type_id), &self.type_id())
                .field(
                    stringify!(free_for_reuse_deadline_ms),
                    &self.free_for_reuse_deadline_ms(),
                )
                .field(stringify!(label_length), &self.label_length())
                .finish()
        }
    }
}
impl AeronCounterMetadataDescriptor {
    #[inline]
    pub fn new(
        state: i32,
        type_id: i32,
        free_for_reuse_deadline_ms: i64,
        key: [u8; 112usize],
        label_length: i32,
        label: [u8; 380usize],
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_counter_metadata_descriptor_t {
                    state: state.into(),
                    type_id: type_id.into(),
                    free_for_reuse_deadline_ms: free_for_reuse_deadline_ms.into(),
                    key: key.into(),
                    label_length: label_length.into(),
                    label: label.into(),
                };
                let inner_ptr: *mut aeron_counter_metadata_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counter_metadata_descriptor_t)
                );
                let inst: aeron_counter_metadata_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counter_metadata_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counter_metadata_descriptor_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn state(&self) -> i32 {
        self.state.into()
    }
    #[inline]
    pub fn type_id(&self) -> i32 {
        self.type_id.into()
    }
    #[inline]
    pub fn free_for_reuse_deadline_ms(&self) -> i64 {
        self.free_for_reuse_deadline_ms.into()
    }
    #[inline]
    pub fn key(&self) -> [u8; 112usize] {
        self.key.into()
    }
    #[inline]
    pub fn label_length(&self) -> i32 {
        self.label_length.into()
    }
    #[inline]
    pub fn label(&self) -> [u8; 380usize] {
        self.label.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counter_metadata_descriptor_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counter_metadata_descriptor_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counter_metadata_descriptor_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCounterMetadataDescriptor {
    type Target = aeron_counter_metadata_descriptor_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counter_metadata_descriptor_t> for AeronCounterMetadataDescriptor {
    #[inline]
    fn from(value: *mut aeron_counter_metadata_descriptor_t) -> Self {
        AeronCounterMetadataDescriptor {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCounterMetadataDescriptor> for *mut aeron_counter_metadata_descriptor_t {
    #[inline]
    fn from(value: AeronCounterMetadataDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCounterMetadataDescriptor> for *mut aeron_counter_metadata_descriptor_t {
    #[inline]
    fn from(value: &AeronCounterMetadataDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronCounterMetadataDescriptor> for aeron_counter_metadata_descriptor_t {
    #[inline]
    fn from(value: AeronCounterMetadataDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counter_metadata_descriptor_t> for AeronCounterMetadataDescriptor {
    #[inline]
    fn from(value: *const aeron_counter_metadata_descriptor_t) -> Self {
        AeronCounterMetadataDescriptor {
            inner: CResource::Borrowed(value as *mut aeron_counter_metadata_descriptor_t),
        }
    }
}
impl From<aeron_counter_metadata_descriptor_t> for AeronCounterMetadataDescriptor {
    #[inline]
    fn from(value: aeron_counter_metadata_descriptor_t) -> Self {
        AeronCounterMetadataDescriptor {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCounterMetadataDescriptor {
    fn default() -> Self {
        AeronCounterMetadataDescriptor::new_zeroed_on_heap()
    }
}
impl AeronCounterMetadataDescriptor {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCounter {
    inner: CResource<aeron_counter_t>,
}
impl core::fmt::Debug for AeronCounter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCounter))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCounter))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronCounter {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counter_t)
                );
                let inst: aeron_counter_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            Some(|c| unsafe { aeron_counter_is_closed(c) }),
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counter_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Return a pointer to the counter value."]
    #[doc = ""]
    #[doc = " \n# Return\n pointer to the counter value."]
    pub fn addr(&self) -> &mut i64 {
        unsafe {
            let result = aeron_counter_addr(self.get_inner());
            unsafe { &mut *result }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a counter."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `counter` to get the constants for."]
    #[doc = " \n - `constants` structure to fill in with the constants."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn constants(&self, constants: &AeronCounterConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_counter_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a counter."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronCounterConstants, AeronCError> {
        let result = AeronCounterConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Asynchronously close the counter."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn close<AeronNotificationHandlerImpl: AeronNotificationCallback>(
        &self,
        on_close_complete: Option<&Handler<AeronNotificationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_counter_close(
                self.get_inner(),
                {
                    let callback: aeron_notification_t = if on_close_complete.is_none() {
                        None
                    } else {
                        Some(aeron_notification_t_callback::<AeronNotificationHandlerImpl>)
                    };
                    callback
                },
                on_close_complete
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Asynchronously close the counter."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn close_once<AeronNotificationHandlerImpl: FnMut() -> ()>(
        &self,
        mut on_close_complete: AeronNotificationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_counter_close(
                self.get_inner(),
                Some(
                    aeron_notification_t_callback_for_once_closure::<AeronNotificationHandlerImpl>,
                ),
                &mut on_close_complete as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Check if the counter is closed"]
    #[doc = " \n# Return\n true if closed, false otherwise."]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_counter_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counter_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counter_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counter_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCounter {
    type Target = aeron_counter_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counter_t> for AeronCounter {
    #[inline]
    fn from(value: *mut aeron_counter_t) -> Self {
        AeronCounter {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCounter> for *mut aeron_counter_t {
    #[inline]
    fn from(value: AeronCounter) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCounter> for *mut aeron_counter_t {
    #[inline]
    fn from(value: &AeronCounter) -> Self {
        value.get_inner()
    }
}
impl From<AeronCounter> for aeron_counter_t {
    #[inline]
    fn from(value: AeronCounter) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counter_t> for AeronCounter {
    #[inline]
    fn from(value: *const aeron_counter_t) -> Self {
        AeronCounter {
            inner: CResource::Borrowed(value as *mut aeron_counter_t),
        }
    }
}
impl From<aeron_counter_t> for AeronCounter {
    #[inline]
    fn from(value: aeron_counter_t) -> Self {
        AeronCounter {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronCounter {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronCounter));
                    let result = self.close_with_no_args();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronCounter));
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct AeronCounterValueDescriptor {
    inner: CResource<aeron_counter_value_descriptor_t>,
}
impl core::fmt::Debug for AeronCounterValueDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCounterValueDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCounterValueDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(counter_value), &self.counter_value())
                .field(stringify!(registration_id), &self.registration_id())
                .field(stringify!(owner_id), &self.owner_id())
                .field(stringify!(reference_id), &self.reference_id())
                .finish()
        }
    }
}
impl AeronCounterValueDescriptor {
    #[inline]
    pub fn new(
        counter_value: i64,
        registration_id: i64,
        owner_id: i64,
        reference_id: i64,
        pad1: [u8; 96usize],
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_counter_value_descriptor_t {
                    counter_value: counter_value.into(),
                    registration_id: registration_id.into(),
                    owner_id: owner_id.into(),
                    reference_id: reference_id.into(),
                    pad1: pad1.into(),
                };
                let inner_ptr: *mut aeron_counter_value_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counter_value_descriptor_t)
                );
                let inst: aeron_counter_value_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counter_value_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counter_value_descriptor_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn counter_value(&self) -> i64 {
        self.counter_value.into()
    }
    #[inline]
    pub fn registration_id(&self) -> i64 {
        self.registration_id.into()
    }
    #[inline]
    pub fn owner_id(&self) -> i64 {
        self.owner_id.into()
    }
    #[inline]
    pub fn reference_id(&self) -> i64 {
        self.reference_id.into()
    }
    #[inline]
    pub fn pad1(&self) -> [u8; 96usize] {
        self.pad1.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counter_value_descriptor_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counter_value_descriptor_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counter_value_descriptor_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCounterValueDescriptor {
    type Target = aeron_counter_value_descriptor_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counter_value_descriptor_t> for AeronCounterValueDescriptor {
    #[inline]
    fn from(value: *mut aeron_counter_value_descriptor_t) -> Self {
        AeronCounterValueDescriptor {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCounterValueDescriptor> for *mut aeron_counter_value_descriptor_t {
    #[inline]
    fn from(value: AeronCounterValueDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCounterValueDescriptor> for *mut aeron_counter_value_descriptor_t {
    #[inline]
    fn from(value: &AeronCounterValueDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronCounterValueDescriptor> for aeron_counter_value_descriptor_t {
    #[inline]
    fn from(value: AeronCounterValueDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counter_value_descriptor_t> for AeronCounterValueDescriptor {
    #[inline]
    fn from(value: *const aeron_counter_value_descriptor_t) -> Self {
        AeronCounterValueDescriptor {
            inner: CResource::Borrowed(value as *mut aeron_counter_value_descriptor_t),
        }
    }
}
impl From<aeron_counter_value_descriptor_t> for AeronCounterValueDescriptor {
    #[inline]
    fn from(value: aeron_counter_value_descriptor_t) -> Self {
        AeronCounterValueDescriptor {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCounterValueDescriptor {
    fn default() -> Self {
        AeronCounterValueDescriptor::new_zeroed_on_heap()
    }
}
impl AeronCounterValueDescriptor {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCountersReaderBuffers {
    inner: CResource<aeron_counters_reader_buffers_t>,
}
impl core::fmt::Debug for AeronCountersReaderBuffers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCountersReaderBuffers))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCountersReaderBuffers))
                .field("inner", &self.inner)
                .field(stringify!(values_length), &self.values_length())
                .field(stringify!(metadata_length), &self.metadata_length())
                .finish()
        }
    }
}
impl AeronCountersReaderBuffers {
    #[inline]
    pub fn new(
        values: *mut u8,
        metadata: *mut u8,
        values_length: usize,
        metadata_length: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_counters_reader_buffers_t {
                    values: values.into(),
                    metadata: metadata.into(),
                    values_length: values_length.into(),
                    metadata_length: metadata_length.into(),
                };
                let inner_ptr: *mut aeron_counters_reader_buffers_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counters_reader_buffers_t)
                );
                let inst: aeron_counters_reader_buffers_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counters_reader_buffers_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counters_reader_buffers_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn values(&self) -> *mut u8 {
        self.values.into()
    }
    #[inline]
    pub fn metadata(&self) -> *mut u8 {
        self.metadata.into()
    }
    #[inline]
    pub fn values_length(&self) -> usize {
        self.values_length.into()
    }
    #[inline]
    pub fn metadata_length(&self) -> usize {
        self.metadata_length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counters_reader_buffers_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counters_reader_buffers_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counters_reader_buffers_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCountersReaderBuffers {
    type Target = aeron_counters_reader_buffers_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counters_reader_buffers_t> for AeronCountersReaderBuffers {
    #[inline]
    fn from(value: *mut aeron_counters_reader_buffers_t) -> Self {
        AeronCountersReaderBuffers {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCountersReaderBuffers> for *mut aeron_counters_reader_buffers_t {
    #[inline]
    fn from(value: AeronCountersReaderBuffers) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCountersReaderBuffers> for *mut aeron_counters_reader_buffers_t {
    #[inline]
    fn from(value: &AeronCountersReaderBuffers) -> Self {
        value.get_inner()
    }
}
impl From<AeronCountersReaderBuffers> for aeron_counters_reader_buffers_t {
    #[inline]
    fn from(value: AeronCountersReaderBuffers) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counters_reader_buffers_t> for AeronCountersReaderBuffers {
    #[inline]
    fn from(value: *const aeron_counters_reader_buffers_t) -> Self {
        AeronCountersReaderBuffers {
            inner: CResource::Borrowed(value as *mut aeron_counters_reader_buffers_t),
        }
    }
}
impl From<aeron_counters_reader_buffers_t> for AeronCountersReaderBuffers {
    #[inline]
    fn from(value: aeron_counters_reader_buffers_t) -> Self {
        AeronCountersReaderBuffers {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCountersReaderBuffers {
    fn default() -> Self {
        AeronCountersReaderBuffers::new_zeroed_on_heap()
    }
}
impl AeronCountersReaderBuffers {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCountersReader {
    inner: CResource<aeron_counters_reader_t>,
}
impl core::fmt::Debug for AeronCountersReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCountersReader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCountersReader))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronCountersReader {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_counters_reader_t)
                );
                let inst: aeron_counters_reader_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_counters_reader_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_counters_reader_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Get buffer pointers and lengths for the counters reader."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `reader` reader containing the buffers."]
    #[doc = " \n - `buffers` output structure to return the buffers."]
    #[doc = " \n# Return\n -1 on failure, 0 on success."]
    pub fn get_buffers(&self, buffers: &AeronCountersReaderBuffers) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_counters_reader_get_buffers(self.get_inner(), buffers.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Iterate over the counters in the counters_reader and call the given function for each counter."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `func` to call for each counter."]
    #[doc = " \n - `clientd` to pass for each call to func."]
    pub fn foreach_counter<
        AeronCountersReaderForeachCounterFuncHandlerImpl: AeronCountersReaderForeachCounterFuncCallback,
    >(
        &self,
        func: Option<&Handler<AeronCountersReaderForeachCounterFuncHandlerImpl>>,
    ) -> () {
        unsafe {
            let result = aeron_counters_reader_foreach_counter(
                self.get_inner(),
                {
                    let callback: aeron_counters_reader_foreach_counter_func_t = if func.is_none() {
                        None
                    } else {
                        Some(
                            aeron_counters_reader_foreach_counter_func_t_callback::<
                                AeronCountersReaderForeachCounterFuncHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                func.map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Iterate over the counters in the counters_reader and call the given function for each counter."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `func` to call for each counter."]
    #[doc = " \n - `clientd` to pass for each call to func."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn foreach_counter_once<
        AeronCountersReaderForeachCounterFuncHandlerImpl: FnMut(i64, i32, i32, &[u8], &str) -> (),
    >(
        &self,
        mut func: AeronCountersReaderForeachCounterFuncHandlerImpl,
    ) -> () {
        unsafe {
            let result = aeron_counters_reader_foreach_counter(
                self.get_inner(),
                Some(
                    aeron_counters_reader_foreach_counter_func_t_callback_for_once_closure::<
                        AeronCountersReaderForeachCounterFuncHandlerImpl,
                    >,
                ),
                &mut func as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Iterate over allocated counters and find the first matching a given type id and registration id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `type_id` to find."]
    #[doc = " \n - `registration_id` to find."]
    #[doc = " \n# Return\n the counter id if found otherwise AERON_NULL_COUNTER_ID."]
    pub fn find_by_type_id_and_registration_id(&self, type_id: i32, registration_id: i64) -> i32 {
        unsafe {
            let result = aeron_counters_reader_find_by_type_id_and_registration_id(
                self.get_inner(),
                type_id.into(),
                registration_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the current max counter id."]
    #[doc = ""]
    #[doc = " \n# Return\n -1 on failure, max counter id on success."]
    pub fn max_counter_id(&self) -> i32 {
        unsafe {
            let result = aeron_counters_reader_max_counter_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the address for a counter."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `counter_id` to find"]
    #[doc = " \n# Return\n address of the counter value"]
    pub fn addr(&self, counter_id: i32) -> &mut i64 {
        unsafe {
            let result = aeron_counters_reader_addr(self.get_inner(), counter_id.into());
            unsafe { &mut *result }
        }
    }
    #[inline]
    #[doc = "Get the registration id assigned to a counter."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id`      for which the registration id is requested."]
    #[doc = " \n - `registration_id` pointer for value to be set on success."]
    pub fn counter_registration_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_counters_reader_counter_registration_id(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Get the owner id assigned to a counter which will typically be the client id."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id`      for which the owner id is requested."]
    #[doc = " \n - `owner_id`        pointer for value to be set on success."]
    pub fn counter_owner_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_counters_reader_counter_owner_id(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Get the reference id assigned to a counter which will typically be the registration id of an associated Image,"]
    #[doc = " Subscription, Publication, etc."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id`      for which the reference id is requested."]
    #[doc = " \n - `reference_id`    pointer for value to be set on success."]
    pub fn counter_reference_id(&self, counter_id: i32) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_counters_reader_counter_reference_id(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Get the state for a counter."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id` to find"]
    #[doc = " \n - `state` out pointer for the current state to be stored in."]
    pub fn counter_state(&self, counter_id: i32) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_counters_reader_counter_state(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Get the type id for a counter."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id` to find"]
    pub fn counter_type_id(&self, counter_id: i32) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_counters_reader_counter_type_id(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Get the label for a counter."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `counter_id` to find"]
    #[doc = " \n - `buffer` to store the counter in."]
    #[doc = " \n - `buffer_length` length of the output buffer"]
    #[doc = " \n# Return\n -1 on failure, number of characters copied to buffer on success."]
    pub fn counter_label(
        &self,
        counter_id: i32,
        buffer: *mut ::std::os::raw::c_char,
        buffer_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_counters_reader_counter_label(
                self.get_inner(),
                counter_id.into(),
                buffer.into(),
                buffer_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Get the free for reuse deadline (ms) for a counter."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id` to find."]
    #[doc = " \n - `deadline_ms` output value to store the deadline."]
    pub fn free_for_reuse_deadline_ms(&self, counter_id: i32) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_counters_reader_free_for_reuse_deadline_ms(
                self.get_inner(),
                counter_id.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Find the active counter id for a stream based on the recording id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `recording_id` the recording id of an active recording"]
    #[doc = " \n# Return\n the counter id if found, otherwise AERON_NULL_COUNTER_ID"]
    pub fn aeron_archive_recording_pos_find_counter_id_by_recording_id(
        &self,
        recording_id: i64,
    ) -> i32 {
        unsafe {
            let result = aeron_archive_recording_pos_find_counter_id_by_recording_id(
                self.get_inner(),
                recording_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Find the active counter id for a stream based on the session id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `session_id` the session id of an active recording"]
    #[doc = " \n# Return\n the counter id if found, otherwise AERON_NULL_COUNTER_ID"]
    pub fn aeron_archive_recording_pos_find_counter_id_by_session_id(
        &self,
        session_id: i32,
    ) -> i32 {
        unsafe {
            let result = aeron_archive_recording_pos_find_counter_id_by_session_id(
                self.get_inner(),
                session_id.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the recording id for a given counter id."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `counter_id` the counter id of an active recording"]
    #[doc = " \n# Return\n the recording id if found, otherwise AERON_NULL_COUNTER_ID"]
    pub fn aeron_archive_recording_pos_get_recording_id(&self, counter_id: i32) -> i64 {
        unsafe {
            let result =
                aeron_archive_recording_pos_get_recording_id(self.get_inner(), counter_id.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the source identity for the recording."]
    #[doc = " \n"]
    #[doc = " See source_identity in `AeronImageConstants`."]
    #[doc = ""]
    #[doc = "\n \n # Parameters
- `counter_id` the counter id of an active recording"]
    #[doc = " \n - `dst` a destination buffer into which the source identity will be written"]
    #[doc = " \n - `len_p` a pointer to a size_t that initially indicates the length of the dst buffer.  After the function return successfully, len_p will be set to the length of the source identity string in dst"]
    pub fn aeron_archive_recording_pos_get_source_identity(
        &self,
        counter_id: i32,
        dst: &std::ffi::CStr,
    ) -> Result<usize, AeronCError> {
        unsafe {
            let mut mut_result: usize = Default::default();
            let err_code = aeron_archive_recording_pos_get_source_identity(
                self.get_inner(),
                counter_id.into(),
                dst.as_ptr(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    #[doc = "Is the recording counter still active?"]
    #[doc = ""]
    #[doc = "\n \n 
# Return
`is_active` out param set to true if the counter is still active"]
    #[doc = " \n # Parameters
- `counter_id` the counter id to search for"]
    #[doc = " \n - `recording_id` the recording id to match against"]
    pub fn aeron_archive_recording_pos_is_active(
        &self,
        counter_id: i32,
        recording_id: i64,
    ) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_archive_recording_pos_is_active(
                &mut mut_result,
                self.get_inner(),
                counter_id.into(),
                recording_id.into(),
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_counters_reader_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_counters_reader_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_counters_reader_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCountersReader {
    type Target = aeron_counters_reader_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_counters_reader_t> for AeronCountersReader {
    #[inline]
    fn from(value: *mut aeron_counters_reader_t) -> Self {
        AeronCountersReader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCountersReader> for *mut aeron_counters_reader_t {
    #[inline]
    fn from(value: AeronCountersReader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCountersReader> for *mut aeron_counters_reader_t {
    #[inline]
    fn from(value: &AeronCountersReader) -> Self {
        value.get_inner()
    }
}
impl From<AeronCountersReader> for aeron_counters_reader_t {
    #[inline]
    fn from(value: AeronCountersReader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_counters_reader_t> for AeronCountersReader {
    #[inline]
    fn from(value: *const aeron_counters_reader_t) -> Self {
        AeronCountersReader {
            inner: CResource::Borrowed(value as *mut aeron_counters_reader_t),
        }
    }
}
impl From<aeron_counters_reader_t> for AeronCountersReader {
    #[inline]
    fn from(value: aeron_counters_reader_t) -> Self {
        AeronCountersReader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronDataHeaderAsLongs {
    inner: CResource<aeron_data_header_as_longs_t>,
}
impl core::fmt::Debug for AeronDataHeaderAsLongs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronDataHeaderAsLongs))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronDataHeaderAsLongs))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronDataHeaderAsLongs {
    #[inline]
    pub fn new(hdr: [u64; 4usize]) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_data_header_as_longs_t { hdr: hdr.into() };
                let inner_ptr: *mut aeron_data_header_as_longs_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_data_header_as_longs_t)
                );
                let inst: aeron_data_header_as_longs_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_data_header_as_longs_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_data_header_as_longs_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn hdr(&self) -> [u64; 4usize] {
        self.hdr.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_data_header_as_longs_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_data_header_as_longs_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_data_header_as_longs_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronDataHeaderAsLongs {
    type Target = aeron_data_header_as_longs_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_data_header_as_longs_t> for AeronDataHeaderAsLongs {
    #[inline]
    fn from(value: *mut aeron_data_header_as_longs_t) -> Self {
        AeronDataHeaderAsLongs {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronDataHeaderAsLongs> for *mut aeron_data_header_as_longs_t {
    #[inline]
    fn from(value: AeronDataHeaderAsLongs) -> Self {
        value.get_inner()
    }
}
impl From<&AeronDataHeaderAsLongs> for *mut aeron_data_header_as_longs_t {
    #[inline]
    fn from(value: &AeronDataHeaderAsLongs) -> Self {
        value.get_inner()
    }
}
impl From<AeronDataHeaderAsLongs> for aeron_data_header_as_longs_t {
    #[inline]
    fn from(value: AeronDataHeaderAsLongs) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_data_header_as_longs_t> for AeronDataHeaderAsLongs {
    #[inline]
    fn from(value: *const aeron_data_header_as_longs_t) -> Self {
        AeronDataHeaderAsLongs {
            inner: CResource::Borrowed(value as *mut aeron_data_header_as_longs_t),
        }
    }
}
impl From<aeron_data_header_as_longs_t> for AeronDataHeaderAsLongs {
    #[inline]
    fn from(value: aeron_data_header_as_longs_t) -> Self {
        AeronDataHeaderAsLongs {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronDataHeaderAsLongs {
    fn default() -> Self {
        AeronDataHeaderAsLongs::new_zeroed_on_heap()
    }
}
impl AeronDataHeaderAsLongs {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronDataHeader {
    inner: CResource<aeron_data_header_t>,
}
impl core::fmt::Debug for AeronDataHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronDataHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronDataHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(term_offset), &self.term_offset())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(term_id), &self.term_id())
                .field(stringify!(reserved_value), &self.reserved_value())
                .finish()
        }
    }
}
impl AeronDataHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        term_offset: i32,
        session_id: i32,
        stream_id: i32,
        term_id: i32,
        reserved_value: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_data_header_t {
                    frame_header: frame_header.into(),
                    term_offset: term_offset.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    term_id: term_id.into(),
                    reserved_value: reserved_value.into(),
                };
                let inner_ptr: *mut aeron_data_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_data_header_t)
                );
                let inst: aeron_data_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_data_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_data_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn term_offset(&self) -> i32 {
        self.term_offset.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn term_id(&self) -> i32 {
        self.term_id.into()
    }
    #[inline]
    pub fn reserved_value(&self) -> i64 {
        self.reserved_value.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_data_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_data_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_data_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronDataHeader {
    type Target = aeron_data_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_data_header_t> for AeronDataHeader {
    #[inline]
    fn from(value: *mut aeron_data_header_t) -> Self {
        AeronDataHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronDataHeader> for *mut aeron_data_header_t {
    #[inline]
    fn from(value: AeronDataHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronDataHeader> for *mut aeron_data_header_t {
    #[inline]
    fn from(value: &AeronDataHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronDataHeader> for aeron_data_header_t {
    #[inline]
    fn from(value: AeronDataHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_data_header_t> for AeronDataHeader {
    #[inline]
    fn from(value: *const aeron_data_header_t) -> Self {
        AeronDataHeader {
            inner: CResource::Borrowed(value as *mut aeron_data_header_t),
        }
    }
}
impl From<aeron_data_header_t> for AeronDataHeader {
    #[inline]
    fn from(value: aeron_data_header_t) -> Self {
        AeronDataHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronDataHeader {
    fn default() -> Self {
        AeronDataHeader::new_zeroed_on_heap()
    }
}
impl AeronDataHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronError {
    inner: CResource<aeron_error_t>,
}
impl core::fmt::Debug for AeronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronError))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronError))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(receiver_id), &self.receiver_id())
                .field(stringify!(group_tag), &self.group_tag())
                .field(stringify!(error_code), &self.error_code())
                .field(stringify!(error_length), &self.error_length())
                .finish()
        }
    }
}
impl AeronError {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        session_id: i32,
        stream_id: i32,
        receiver_id: i64,
        group_tag: i64,
        error_code: i32,
        error_length: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_error_t {
                    frame_header: frame_header.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    receiver_id: receiver_id.into(),
                    group_tag: group_tag.into(),
                    error_code: error_code.into(),
                    error_length: error_length.into(),
                };
                let inner_ptr: *mut aeron_error_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_error_t)
                );
                let inst: aeron_error_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_error_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_error_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn receiver_id(&self) -> i64 {
        self.receiver_id.into()
    }
    #[inline]
    pub fn group_tag(&self) -> i64 {
        self.group_tag.into()
    }
    #[inline]
    pub fn error_code(&self) -> i32 {
        self.error_code.into()
    }
    #[inline]
    pub fn error_length(&self) -> i32 {
        self.error_length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_error_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_error_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_error_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronError {
    type Target = aeron_error_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_error_t> for AeronError {
    #[inline]
    fn from(value: *mut aeron_error_t) -> Self {
        AeronError {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronError> for *mut aeron_error_t {
    #[inline]
    fn from(value: AeronError) -> Self {
        value.get_inner()
    }
}
impl From<&AeronError> for *mut aeron_error_t {
    #[inline]
    fn from(value: &AeronError) -> Self {
        value.get_inner()
    }
}
impl From<AeronError> for aeron_error_t {
    #[inline]
    fn from(value: AeronError) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_error_t> for AeronError {
    #[inline]
    fn from(value: *const aeron_error_t) -> Self {
        AeronError {
            inner: CResource::Borrowed(value as *mut aeron_error_t),
        }
    }
}
impl From<aeron_error_t> for AeronError {
    #[inline]
    fn from(value: aeron_error_t) -> Self {
        AeronError {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronError {
    fn default() -> Self {
        AeronError::new_zeroed_on_heap()
    }
}
impl AeronError {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronExclusivePublication {
    inner: CResource<aeron_exclusive_publication_t>,
}
impl core::fmt::Debug for AeronExclusivePublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronExclusivePublication))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronExclusivePublication))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronExclusivePublication {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_exclusive_publication_t)
                );
                let inst: aeron_exclusive_publication_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_exclusive_publication_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            Some(|c| unsafe { aeron_exclusive_publication_is_closed(c) }),
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_exclusive_publication_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Non-blocking publish of a buffer containing a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` to publish."]
    #[doc = " \n - `length` of the buffer."]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn offer<AeronReservedValueSupplierHandlerImpl: AeronReservedValueSupplierCallback>(
        &self,
        buffer: &[u8],
        reserved_value_supplier: Option<&Handler<AeronReservedValueSupplierHandlerImpl>>,
    ) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_offer(
                self.get_inner(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                {
                    let callback: aeron_reserved_value_supplier_t =
                        if reserved_value_supplier.is_none() {
                            None
                        } else {
                            Some(
                                aeron_reserved_value_supplier_t_callback::<
                                    AeronReservedValueSupplierHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                reserved_value_supplier
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish of a buffer containing a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` to publish."]
    #[doc = " \n - `length` of the buffer."]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn offer_once<AeronReservedValueSupplierHandlerImpl: FnMut(*mut u8, usize) -> i64>(
        &self,
        buffer: &[u8],
        mut reserved_value_supplier: AeronReservedValueSupplierHandlerImpl,
    ) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_offer(
                self.get_inner(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                Some(
                    aeron_reserved_value_supplier_t_callback_for_once_closure::<
                        AeronReservedValueSupplierHandlerImpl,
                    >,
                ),
                &mut reserved_value_supplier as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish by gathering buffer vectors into a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `iov` array for the vectors"]
    #[doc = " \n - `iovcnt` of the number of vectors"]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn offerv<AeronReservedValueSupplierHandlerImpl: AeronReservedValueSupplierCallback>(
        &self,
        iov: &AeronIovec,
        iovcnt: usize,
        reserved_value_supplier: Option<&Handler<AeronReservedValueSupplierHandlerImpl>>,
    ) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_offerv(
                self.get_inner(),
                iov.get_inner(),
                iovcnt.into(),
                {
                    let callback: aeron_reserved_value_supplier_t =
                        if reserved_value_supplier.is_none() {
                            None
                        } else {
                            Some(
                                aeron_reserved_value_supplier_t_callback::<
                                    AeronReservedValueSupplierHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                reserved_value_supplier
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish by gathering buffer vectors into a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `iov` array for the vectors"]
    #[doc = " \n - `iovcnt` of the number of vectors"]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn offerv_once<AeronReservedValueSupplierHandlerImpl: FnMut(*mut u8, usize) -> i64>(
        &self,
        iov: &AeronIovec,
        iovcnt: usize,
        mut reserved_value_supplier: AeronReservedValueSupplierHandlerImpl,
    ) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_offerv(
                self.get_inner(),
                iov.get_inner(),
                iovcnt.into(),
                Some(
                    aeron_reserved_value_supplier_t_callback_for_once_closure::<
                        AeronReservedValueSupplierHandlerImpl,
                    >,
                ),
                &mut reserved_value_supplier as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Try to claim a range in the publication log into which a message can be written with zero copy semantics."]
    #[doc = " Once the message has been written then aeron_buffer_claim_commit should be called thus making it available."]
    #[doc = " A claim length cannot be greater than max payload length."]
    #[doc = " \n"]
    #[doc = " <b>Note:</b> This method can only be used for message lengths less than MTU length minus header."]
    #[doc = ""]
    #[doc = " @code"]
    #[doc = " `AeronBufferClaim` buffer_claim;"]
    #[doc = ""]
    #[doc = " if (`AeronExclusivePublication`ry_claim(publication, length, &buffer_claim) > 0L)"]
    #[doc = " {"]
    #[doc = "     // work with buffer_claim->data directly."]
    #[doc = "     aeron_buffer_claim_commit(&buffer_claim);"]
    #[doc = " }"]
    #[doc = " @endcode"]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `length` of the message."]
    #[doc = " \n - `buffer_claim` to be populated if the claim succeeds."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn try_claim(&self, length: usize, buffer_claim: &AeronBufferClaim) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_try_claim(
                self.get_inner(),
                length.into(),
                buffer_claim.get_inner(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Append a padding record log of a given length to make up the log to a position."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `length` of the range to claim, in bytes."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn append_padding(&self, length: usize) -> i64 {
        unsafe {
            let result =
                aeron_exclusive_publication_append_padding(self.get_inner(), length.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Offer a block of pre-formatted message fragments directly into the current term."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` containing the pre-formatted block of message fragments."]
    #[doc = " \n - `offset` offset in the buffer at which the first fragment begins."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn offer_block(&self, buffer: &[u8]) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_offer_block(
                self.get_inner(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the status of the media channel for this publication."]
    #[doc = " \n"]
    #[doc = " The status will be ERRORED (-1) if a socket exception occurs on setup and ACTIVE (1) if all is well."]
    #[doc = ""]
    #[doc = " \n# Return\n 1 for ACTIVE, -1 for ERRORED"]
    pub fn channel_status(&self) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_channel_status(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `publication` to get the constants for."]
    #[doc = " \n - `constants` structure to fill in with the constants"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn constants(&self, constants: &AeronPublicationConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_exclusive_publication_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a publication."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronPublicationConstants, AeronCError> {
        let result = AeronPublicationConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the current position to which the publication has advanced for this stream."]
    #[doc = ""]
    #[doc = " \n# Return\n the current position to which the publication has advanced for this stream or a negative error value."]
    pub fn position(&self) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_position(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the position limit beyond which this publication will be back pressured."]
    #[doc = ""]
    #[doc = " This should only be used as a guide to determine when back pressure is likely to be applied."]
    #[doc = ""]
    #[doc = " \n# Return\n the position limit beyond which this publication will be back pressured or a negative error value."]
    pub fn position_limit(&self) -> i64 {
        unsafe {
            let result = aeron_exclusive_publication_position_limit(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Asynchronously close the publication."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn close<AeronNotificationHandlerImpl: AeronNotificationCallback>(
        &self,
        on_close_complete: Option<&Handler<AeronNotificationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_exclusive_publication_close(
                self.get_inner(),
                {
                    let callback: aeron_notification_t = if on_close_complete.is_none() {
                        None
                    } else {
                        Some(aeron_notification_t_callback::<AeronNotificationHandlerImpl>)
                    };
                    callback
                },
                on_close_complete
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Asynchronously close the publication."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn close_once<AeronNotificationHandlerImpl: FnMut() -> ()>(
        &self,
        mut on_close_complete: AeronNotificationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_exclusive_publication_close(
                self.get_inner(),
                Some(
                    aeron_notification_t_callback_for_once_closure::<AeronNotificationHandlerImpl>,
                ),
                &mut on_close_complete as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Revoke this publication when it's closed."]
    #[doc = ""]
    pub fn revoke_on_close(&self) -> () {
        unsafe {
            let result = aeron_exclusive_publication_revoke_on_close(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Asynchronously revoke and close the publication. Will callback on the on_complete notification when the publicaiton is closed."]
    #[doc = " The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the publication has been revoked, closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn revoke<AeronNotificationHandlerImpl: AeronNotificationCallback>(
        &self,
        on_close_complete: Option<&Handler<AeronNotificationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_exclusive_publication_revoke(
                self.get_inner(),
                {
                    let callback: aeron_notification_t = if on_close_complete.is_none() {
                        None
                    } else {
                        Some(aeron_notification_t_callback::<AeronNotificationHandlerImpl>)
                    };
                    callback
                },
                on_close_complete
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Asynchronously revoke and close the publication. Will callback on the on_complete notification when the publicaiton is closed."]
    #[doc = " The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the publication has been revoked, closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn revoke_once<AeronNotificationHandlerImpl: FnMut() -> ()>(
        &self,
        mut on_close_complete: AeronNotificationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_exclusive_publication_revoke(
                self.get_inner(),
                Some(
                    aeron_notification_t_callback_for_once_closure::<AeronNotificationHandlerImpl>,
                ),
                &mut on_close_complete as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Has the exclusive publication closed?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if this publication is closed."]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_exclusive_publication_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Has the exclusive publication seen an active Subscriber recently?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if this publication has recently seen an active subscriber otherwise false."]
    pub fn is_connected(&self) -> bool {
        unsafe {
            let result = aeron_exclusive_publication_is_connected(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get all of the local socket addresses for this exclusive publication. Typically only one representing the control"]
    #[doc = " address."]
    #[doc = ""]
    #[doc = " @see aeron_subscription_local_sockaddrs"]
    #[doc = "# Parameters\n \n - `address_vec` to hold the received addresses"]
    #[doc = " \n - `address_vec_len` available length of the vector to hold the addresses"]
    #[doc = " \n# Return\n number of addresses found or -1 if there is an error."]
    pub fn local_sockaddrs(
        &self,
        address_vec: &AeronIovec,
        address_vec_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_exclusive_publication_local_sockaddrs(
                self.get_inner(),
                address_vec.get_inner(),
                address_vec_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_exclusive_publication_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_exclusive_publication_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_exclusive_publication_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronExclusivePublication {
    type Target = aeron_exclusive_publication_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_exclusive_publication_t> for AeronExclusivePublication {
    #[inline]
    fn from(value: *mut aeron_exclusive_publication_t) -> Self {
        AeronExclusivePublication {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronExclusivePublication> for *mut aeron_exclusive_publication_t {
    #[inline]
    fn from(value: AeronExclusivePublication) -> Self {
        value.get_inner()
    }
}
impl From<&AeronExclusivePublication> for *mut aeron_exclusive_publication_t {
    #[inline]
    fn from(value: &AeronExclusivePublication) -> Self {
        value.get_inner()
    }
}
impl From<AeronExclusivePublication> for aeron_exclusive_publication_t {
    #[inline]
    fn from(value: AeronExclusivePublication) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_exclusive_publication_t> for AeronExclusivePublication {
    #[inline]
    fn from(value: *const aeron_exclusive_publication_t) -> Self {
        AeronExclusivePublication {
            inner: CResource::Borrowed(value as *mut aeron_exclusive_publication_t),
        }
    }
}
impl From<aeron_exclusive_publication_t> for AeronExclusivePublication {
    #[inline]
    fn from(value: aeron_exclusive_publication_t) -> Self {
        AeronExclusivePublication {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronExclusivePublication {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronExclusivePublication));
                    let result = self.close_with_no_args();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronExclusivePublication));
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct AeronFragmentAssembler {
    inner: CResource<aeron_fragment_assembler_t>,
}
impl core::fmt::Debug for AeronFragmentAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronFragmentAssembler))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronFragmentAssembler))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronFragmentAssembler {
    #[doc = "Create a fragment assembler for use with a subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `delegate` to call on completed"]
    #[doc = " \n - `delegate_clientd` to pass to delegate handler."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        delegate: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let (delegate, delegate_clientd) = (
            {
                let callback: aeron_fragment_handler_t = if delegate.is_none() {
                    None
                } else {
                    Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                };
                callback
            },
            delegate
                .map(|m| m.as_raw())
                .unwrap_or_else(|| std::ptr::null_mut()),
        );
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_fragment_assembler_create(ctx_field, delegate, delegate_clientd)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_fragment_assembler_delete(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Delete a fragment assembler."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn delete(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_fragment_assembler_delete(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Handler function to be passed for handling fragment assembly."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `clientd` passed in the poll call (must be a `AeronFragmentAssembler`)"]
    #[doc = " \n - `buffer` containing the data."]
    #[doc = " \n - `header` representing the meta data for the data."]
    pub fn handler(
        clientd: *mut ::std::os::raw::c_void,
        buffer: &[u8],
        header: &AeronHeader,
    ) -> () {
        unsafe {
            let result = aeron_fragment_assembler_handler(
                clientd.into(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                header.get_inner(),
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_fragment_assembler_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_fragment_assembler_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_fragment_assembler_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronFragmentAssembler {
    type Target = aeron_fragment_assembler_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_fragment_assembler_t> for AeronFragmentAssembler {
    #[inline]
    fn from(value: *mut aeron_fragment_assembler_t) -> Self {
        AeronFragmentAssembler {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronFragmentAssembler> for *mut aeron_fragment_assembler_t {
    #[inline]
    fn from(value: AeronFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<&AeronFragmentAssembler> for *mut aeron_fragment_assembler_t {
    #[inline]
    fn from(value: &AeronFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<AeronFragmentAssembler> for aeron_fragment_assembler_t {
    #[inline]
    fn from(value: AeronFragmentAssembler) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_fragment_assembler_t> for AeronFragmentAssembler {
    #[inline]
    fn from(value: *const aeron_fragment_assembler_t) -> Self {
        AeronFragmentAssembler {
            inner: CResource::Borrowed(value as *mut aeron_fragment_assembler_t),
        }
    }
}
impl From<aeron_fragment_assembler_t> for AeronFragmentAssembler {
    #[inline]
    fn from(value: aeron_fragment_assembler_t) -> Self {
        AeronFragmentAssembler {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronFrameHeader {
    inner: CResource<aeron_frame_header_t>,
}
impl core::fmt::Debug for AeronFrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronFrameHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronFrameHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_length), &self.frame_length())
                .field(stringify!(type_), &self.type_())
                .finish()
        }
    }
}
impl AeronFrameHeader {
    #[inline]
    pub fn new(frame_length: i32, version: i8, flags: u8, type_: i16) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_frame_header_t {
                    frame_length: frame_length.into(),
                    version: version.into(),
                    flags: flags.into(),
                    type_: type_.into(),
                };
                let inner_ptr: *mut aeron_frame_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_frame_header_t)
                );
                let inst: aeron_frame_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_frame_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_frame_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_length(&self) -> i32 {
        self.frame_length.into()
    }
    #[inline]
    pub fn version(&self) -> i8 {
        self.version.into()
    }
    #[inline]
    pub fn flags(&self) -> u8 {
        self.flags.into()
    }
    #[inline]
    pub fn type_(&self) -> i16 {
        self.type_.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_frame_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_frame_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_frame_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronFrameHeader {
    type Target = aeron_frame_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_frame_header_t> for AeronFrameHeader {
    #[inline]
    fn from(value: *mut aeron_frame_header_t) -> Self {
        AeronFrameHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronFrameHeader> for *mut aeron_frame_header_t {
    #[inline]
    fn from(value: AeronFrameHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronFrameHeader> for *mut aeron_frame_header_t {
    #[inline]
    fn from(value: &AeronFrameHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronFrameHeader> for aeron_frame_header_t {
    #[inline]
    fn from(value: AeronFrameHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_frame_header_t> for AeronFrameHeader {
    #[inline]
    fn from(value: *const aeron_frame_header_t) -> Self {
        AeronFrameHeader {
            inner: CResource::Borrowed(value as *mut aeron_frame_header_t),
        }
    }
}
impl From<aeron_frame_header_t> for AeronFrameHeader {
    #[inline]
    fn from(value: aeron_frame_header_t) -> Self {
        AeronFrameHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronFrameHeader {
    fn default() -> Self {
        AeronFrameHeader::new_zeroed_on_heap()
    }
}
impl AeronFrameHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronHeader {
    inner: CResource<aeron_header_t>,
}
impl core::fmt::Debug for AeronHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronHeader))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronHeader {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_header_t)
                );
                let inst: aeron_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Get all of the field values from the header. This will do a memcpy into the supplied header_values_t pointer."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `header` to read values from."]
    #[doc = " \n - `values` to copy values to, must not be null."]
    #[doc = " \n# Return\n 0 on success, -1 on failure."]
    pub fn values(&self, values: &AeronHeaderValues) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_header_values(self.get_inner(), values.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Get all of the field values from the header. This will do a memcpy into the supplied header_values_t pointer."]
    #[doc = ""]
    pub fn get_values(&self) -> Result<AeronHeaderValues, AeronCError> {
        let result = AeronHeaderValues::new_zeroed_on_stack();
        self.values(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the current position to which the Image has advanced on reading this message."]
    #[doc = ""]
    #[doc = " \n# Return\n the current position to which the Image has advanced on reading this message."]
    pub fn position(&self) -> i64 {
        unsafe {
            let result = aeron_header_position(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the number of times to left shift the term count to multiply by term length."]
    #[doc = ""]
    #[doc = " \n# Return\n number of times to left shift the term count to multiply by term length."]
    pub fn position_bits_to_shift(&self) -> usize {
        unsafe {
            let result = aeron_header_position_bits_to_shift(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Calculates the offset of the frame immediately after this one."]
    #[doc = ""]
    #[doc = " \n# Return\n the offset of the next frame."]
    pub fn next_term_offset(&self) -> i32 {
        unsafe {
            let result = aeron_header_next_term_offset(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get a pointer to the context associated with this message. Only valid during poll handling. Is normally a"]
    #[doc = " pointer to an Image instance."]
    #[doc = ""]
    #[doc = " \n# Return\n a pointer to the context associated with this message."]
    pub fn context(&self) -> *mut ::std::os::raw::c_void {
        unsafe {
            let result = aeron_header_context(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronHeader {
    type Target = aeron_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_header_t> for AeronHeader {
    #[inline]
    fn from(value: *mut aeron_header_t) -> Self {
        AeronHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronHeader> for *mut aeron_header_t {
    #[inline]
    fn from(value: AeronHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronHeader> for *mut aeron_header_t {
    #[inline]
    fn from(value: &AeronHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronHeader> for aeron_header_t {
    #[inline]
    fn from(value: AeronHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_header_t> for AeronHeader {
    #[inline]
    fn from(value: *const aeron_header_t) -> Self {
        AeronHeader {
            inner: CResource::Borrowed(value as *mut aeron_header_t),
        }
    }
}
impl From<aeron_header_t> for AeronHeader {
    #[inline]
    fn from(value: aeron_header_t) -> Self {
        AeronHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronHeaderValuesFrame {
    inner: CResource<aeron_header_values_frame_t>,
}
impl core::fmt::Debug for AeronHeaderValuesFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronHeaderValuesFrame))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronHeaderValuesFrame))
                .field("inner", &self.inner)
                .field(stringify!(frame_length), &self.frame_length())
                .field(stringify!(type_), &self.type_())
                .field(stringify!(term_offset), &self.term_offset())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(term_id), &self.term_id())
                .field(stringify!(reserved_value), &self.reserved_value())
                .finish()
        }
    }
}
impl AeronHeaderValuesFrame {
    #[inline]
    pub fn new(
        frame_length: i32,
        version: i8,
        flags: u8,
        type_: i16,
        term_offset: i32,
        session_id: i32,
        stream_id: i32,
        term_id: i32,
        reserved_value: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_header_values_frame_t {
                    frame_length: frame_length.into(),
                    version: version.into(),
                    flags: flags.into(),
                    type_: type_.into(),
                    term_offset: term_offset.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    term_id: term_id.into(),
                    reserved_value: reserved_value.into(),
                };
                let inner_ptr: *mut aeron_header_values_frame_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_header_values_frame_t)
                );
                let inst: aeron_header_values_frame_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_header_values_frame_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_header_values_frame_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_length(&self) -> i32 {
        self.frame_length.into()
    }
    #[inline]
    pub fn version(&self) -> i8 {
        self.version.into()
    }
    #[inline]
    pub fn flags(&self) -> u8 {
        self.flags.into()
    }
    #[inline]
    pub fn type_(&self) -> i16 {
        self.type_.into()
    }
    #[inline]
    pub fn term_offset(&self) -> i32 {
        self.term_offset.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn term_id(&self) -> i32 {
        self.term_id.into()
    }
    #[inline]
    pub fn reserved_value(&self) -> i64 {
        self.reserved_value.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_header_values_frame_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_header_values_frame_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_header_values_frame_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronHeaderValuesFrame {
    type Target = aeron_header_values_frame_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_header_values_frame_t> for AeronHeaderValuesFrame {
    #[inline]
    fn from(value: *mut aeron_header_values_frame_t) -> Self {
        AeronHeaderValuesFrame {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronHeaderValuesFrame> for *mut aeron_header_values_frame_t {
    #[inline]
    fn from(value: AeronHeaderValuesFrame) -> Self {
        value.get_inner()
    }
}
impl From<&AeronHeaderValuesFrame> for *mut aeron_header_values_frame_t {
    #[inline]
    fn from(value: &AeronHeaderValuesFrame) -> Self {
        value.get_inner()
    }
}
impl From<AeronHeaderValuesFrame> for aeron_header_values_frame_t {
    #[inline]
    fn from(value: AeronHeaderValuesFrame) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_header_values_frame_t> for AeronHeaderValuesFrame {
    #[inline]
    fn from(value: *const aeron_header_values_frame_t) -> Self {
        AeronHeaderValuesFrame {
            inner: CResource::Borrowed(value as *mut aeron_header_values_frame_t),
        }
    }
}
impl From<aeron_header_values_frame_t> for AeronHeaderValuesFrame {
    #[inline]
    fn from(value: aeron_header_values_frame_t) -> Self {
        AeronHeaderValuesFrame {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronHeaderValuesFrame {
    fn default() -> Self {
        AeronHeaderValuesFrame::new_zeroed_on_heap()
    }
}
impl AeronHeaderValuesFrame {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronHeaderValues {
    inner: CResource<aeron_header_values_t>,
}
impl core::fmt::Debug for AeronHeaderValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronHeaderValues))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronHeaderValues))
                .field("inner", &self.inner)
                .field(stringify!(frame), &self.frame())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(
                    stringify!(position_bits_to_shift),
                    &self.position_bits_to_shift(),
                )
                .finish()
        }
    }
}
impl AeronHeaderValues {
    #[inline]
    pub fn new(
        frame: AeronHeaderValuesFrame,
        initial_term_id: i32,
        position_bits_to_shift: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_header_values_t {
                    frame: frame.into(),
                    initial_term_id: initial_term_id.into(),
                    position_bits_to_shift: position_bits_to_shift.into(),
                };
                let inner_ptr: *mut aeron_header_values_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_header_values_t)
                );
                let inst: aeron_header_values_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_header_values_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_header_values_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame(&self) -> AeronHeaderValuesFrame {
        self.frame.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn position_bits_to_shift(&self) -> usize {
        self.position_bits_to_shift.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_header_values_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_header_values_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_header_values_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronHeaderValues {
    type Target = aeron_header_values_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_header_values_t> for AeronHeaderValues {
    #[inline]
    fn from(value: *mut aeron_header_values_t) -> Self {
        AeronHeaderValues {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronHeaderValues> for *mut aeron_header_values_t {
    #[inline]
    fn from(value: AeronHeaderValues) -> Self {
        value.get_inner()
    }
}
impl From<&AeronHeaderValues> for *mut aeron_header_values_t {
    #[inline]
    fn from(value: &AeronHeaderValues) -> Self {
        value.get_inner()
    }
}
impl From<AeronHeaderValues> for aeron_header_values_t {
    #[inline]
    fn from(value: AeronHeaderValues) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_header_values_t> for AeronHeaderValues {
    #[inline]
    fn from(value: *const aeron_header_values_t) -> Self {
        AeronHeaderValues {
            inner: CResource::Borrowed(value as *mut aeron_header_values_t),
        }
    }
}
impl From<aeron_header_values_t> for AeronHeaderValues {
    #[inline]
    fn from(value: aeron_header_values_t) -> Self {
        AeronHeaderValues {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronHeaderValues {
    fn default() -> Self {
        AeronHeaderValues::new_zeroed_on_heap()
    }
}
impl AeronHeaderValues {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[doc = "Configuration for an image that does not change during it's lifetime."]
#[derive(Clone)]
pub struct AeronImageConstants {
    inner: CResource<aeron_image_constants_t>,
}
impl core::fmt::Debug for AeronImageConstants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronImageConstants))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronImageConstants))
                .field("inner", &self.inner)
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(stringify!(join_position), &self.join_position())
                .field(
                    stringify!(position_bits_to_shift),
                    &self.position_bits_to_shift(),
                )
                .field(stringify!(term_buffer_length), &self.term_buffer_length())
                .field(stringify!(mtu_length), &self.mtu_length())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(
                    stringify!(subscriber_position_id),
                    &self.subscriber_position_id(),
                )
                .finish()
        }
    }
}
impl AeronImageConstants {
    #[inline]
    pub fn new(
        subscription: &AeronSubscription,
        source_identity: &std::ffi::CStr,
        correlation_id: i64,
        join_position: i64,
        position_bits_to_shift: usize,
        term_buffer_length: usize,
        mtu_length: usize,
        session_id: i32,
        initial_term_id: i32,
        subscriber_position_id: i32,
    ) -> Result<Self, AeronCError> {
        let subscription_copy = subscription.clone();
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_image_constants_t {
                    subscription: subscription.into(),
                    source_identity: source_identity.as_ptr(),
                    correlation_id: correlation_id.into(),
                    join_position: join_position.into(),
                    position_bits_to_shift: position_bits_to_shift.into(),
                    term_buffer_length: term_buffer_length.into(),
                    mtu_length: mtu_length.into(),
                    session_id: session_id.into(),
                    initial_term_id: initial_term_id.into(),
                    subscriber_position_id: subscriber_position_id.into(),
                };
                let inner_ptr: *mut aeron_image_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_image_constants_t)
                );
                let inst: aeron_image_constants_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_image_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_image_constants_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn subscription(&self) -> AeronSubscription {
        self.subscription.into()
    }
    #[inline]
    pub fn source_identity(&self) -> &str {
        if self.source_identity.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.source_identity)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn join_position(&self) -> i64 {
        self.join_position.into()
    }
    #[inline]
    pub fn position_bits_to_shift(&self) -> usize {
        self.position_bits_to_shift.into()
    }
    #[inline]
    pub fn term_buffer_length(&self) -> usize {
        self.term_buffer_length.into()
    }
    #[inline]
    pub fn mtu_length(&self) -> usize {
        self.mtu_length.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn subscriber_position_id(&self) -> i32 {
        self.subscriber_position_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_image_constants_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_image_constants_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_image_constants_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronImageConstants {
    type Target = aeron_image_constants_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_image_constants_t> for AeronImageConstants {
    #[inline]
    fn from(value: *mut aeron_image_constants_t) -> Self {
        AeronImageConstants {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronImageConstants> for *mut aeron_image_constants_t {
    #[inline]
    fn from(value: AeronImageConstants) -> Self {
        value.get_inner()
    }
}
impl From<&AeronImageConstants> for *mut aeron_image_constants_t {
    #[inline]
    fn from(value: &AeronImageConstants) -> Self {
        value.get_inner()
    }
}
impl From<AeronImageConstants> for aeron_image_constants_t {
    #[inline]
    fn from(value: AeronImageConstants) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_image_constants_t> for AeronImageConstants {
    #[inline]
    fn from(value: *const aeron_image_constants_t) -> Self {
        AeronImageConstants {
            inner: CResource::Borrowed(value as *mut aeron_image_constants_t),
        }
    }
}
impl From<aeron_image_constants_t> for AeronImageConstants {
    #[inline]
    fn from(value: aeron_image_constants_t) -> Self {
        AeronImageConstants {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronImageConstants {
    fn default() -> Self {
        AeronImageConstants::new_zeroed_on_heap()
    }
}
impl AeronImageConstants {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronImageControlledFragmentAssembler {
    inner: CResource<aeron_image_controlled_fragment_assembler_t>,
}
impl core::fmt::Debug for AeronImageControlledFragmentAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronImageControlledFragmentAssembler))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronImageControlledFragmentAssembler))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronImageControlledFragmentAssembler {
    #[doc = "Create an image controlled fragment assembler for use with a single image."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `delegate` to call on completed"]
    #[doc = " \n - `delegate_clientd` to pass to delegate handler."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        delegate: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let (delegate, delegate_clientd) = (
            {
                let callback: aeron_controlled_fragment_handler_t = if delegate.is_none() {
                    None
                } else {
                    Some(
                        aeron_controlled_fragment_handler_t_callback::<
                            AeronControlledFragmentHandlerHandlerImpl,
                        >,
                    )
                };
                callback
            },
            delegate
                .map(|m| m.as_raw())
                .unwrap_or_else(|| std::ptr::null_mut()),
        );
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_image_controlled_fragment_assembler_create(
                    ctx_field,
                    delegate,
                    delegate_clientd,
                )
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_image_controlled_fragment_assembler_delete(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Delete an image controlled fragment assembler."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn delete(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_controlled_fragment_assembler_delete(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Handler function to be passed for handling fragment assembly."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `clientd` passed in the poll call (must be a `AeronImageControlledFragmentAssembler`)"]
    #[doc = " \n - `buffer` containing the data."]
    #[doc = " \n - `header` representing the meta data for the data."]
    #[doc = " \n# Return\n The action to be taken with regard to the stream position after the callback."]
    pub fn handler(
        clientd: *mut ::std::os::raw::c_void,
        buffer: &[u8],
        header: &AeronHeader,
    ) -> aeron_controlled_fragment_handler_action_t {
        unsafe {
            let result = aeron_image_controlled_fragment_assembler_handler(
                clientd.into(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                header.get_inner(),
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_image_controlled_fragment_assembler_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_image_controlled_fragment_assembler_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_image_controlled_fragment_assembler_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronImageControlledFragmentAssembler {
    type Target = aeron_image_controlled_fragment_assembler_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_image_controlled_fragment_assembler_t>
    for AeronImageControlledFragmentAssembler
{
    #[inline]
    fn from(value: *mut aeron_image_controlled_fragment_assembler_t) -> Self {
        AeronImageControlledFragmentAssembler {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronImageControlledFragmentAssembler>
    for *mut aeron_image_controlled_fragment_assembler_t
{
    #[inline]
    fn from(value: AeronImageControlledFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<&AeronImageControlledFragmentAssembler>
    for *mut aeron_image_controlled_fragment_assembler_t
{
    #[inline]
    fn from(value: &AeronImageControlledFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<AeronImageControlledFragmentAssembler> for aeron_image_controlled_fragment_assembler_t {
    #[inline]
    fn from(value: AeronImageControlledFragmentAssembler) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_image_controlled_fragment_assembler_t>
    for AeronImageControlledFragmentAssembler
{
    #[inline]
    fn from(value: *const aeron_image_controlled_fragment_assembler_t) -> Self {
        AeronImageControlledFragmentAssembler {
            inner: CResource::Borrowed(value as *mut aeron_image_controlled_fragment_assembler_t),
        }
    }
}
impl From<aeron_image_controlled_fragment_assembler_t> for AeronImageControlledFragmentAssembler {
    #[inline]
    fn from(value: aeron_image_controlled_fragment_assembler_t) -> Self {
        AeronImageControlledFragmentAssembler {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronImageFragmentAssembler {
    inner: CResource<aeron_image_fragment_assembler_t>,
}
impl core::fmt::Debug for AeronImageFragmentAssembler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronImageFragmentAssembler))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronImageFragmentAssembler))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronImageFragmentAssembler {
    #[doc = "Create an image fragment assembler for use with a single image."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `delegate` to call on completed."]
    #[doc = " \n - `delegate_clientd` to pass to delegate handler."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        delegate: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let (delegate, delegate_clientd) = (
            {
                let callback: aeron_fragment_handler_t = if delegate.is_none() {
                    None
                } else {
                    Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                };
                callback
            },
            delegate
                .map(|m| m.as_raw())
                .unwrap_or_else(|| std::ptr::null_mut()),
        );
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe {
                aeron_image_fragment_assembler_create(ctx_field, delegate, delegate_clientd)
            },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_image_fragment_assembler_delete(*ctx_field)
            })),
            false,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
        })
    }
    #[inline]
    #[doc = "Delete an image fragment assembler."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn delete(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_fragment_assembler_delete(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Handler function to be passed for handling fragment assembly."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `clientd` passed in the poll call (must be a `AeronImageFragmentAssembler`)"]
    #[doc = " \n - `buffer` containing the data."]
    #[doc = " \n - `header` representing the meta data for the data."]
    pub fn handler(
        clientd: *mut ::std::os::raw::c_void,
        buffer: &[u8],
        header: &AeronHeader,
    ) -> () {
        unsafe {
            let result = aeron_image_fragment_assembler_handler(
                clientd.into(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                header.get_inner(),
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_image_fragment_assembler_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_image_fragment_assembler_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_image_fragment_assembler_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronImageFragmentAssembler {
    type Target = aeron_image_fragment_assembler_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_image_fragment_assembler_t> for AeronImageFragmentAssembler {
    #[inline]
    fn from(value: *mut aeron_image_fragment_assembler_t) -> Self {
        AeronImageFragmentAssembler {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronImageFragmentAssembler> for *mut aeron_image_fragment_assembler_t {
    #[inline]
    fn from(value: AeronImageFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<&AeronImageFragmentAssembler> for *mut aeron_image_fragment_assembler_t {
    #[inline]
    fn from(value: &AeronImageFragmentAssembler) -> Self {
        value.get_inner()
    }
}
impl From<AeronImageFragmentAssembler> for aeron_image_fragment_assembler_t {
    #[inline]
    fn from(value: AeronImageFragmentAssembler) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_image_fragment_assembler_t> for AeronImageFragmentAssembler {
    #[inline]
    fn from(value: *const aeron_image_fragment_assembler_t) -> Self {
        AeronImageFragmentAssembler {
            inner: CResource::Borrowed(value as *mut aeron_image_fragment_assembler_t),
        }
    }
}
impl From<aeron_image_fragment_assembler_t> for AeronImageFragmentAssembler {
    #[inline]
    fn from(value: aeron_image_fragment_assembler_t) -> Self {
        AeronImageFragmentAssembler {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronImage {
    inner: CResource<aeron_image_t>,
}
impl core::fmt::Debug for AeronImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronImage))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronImage))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronImage {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_image_t)
                );
                let inst: aeron_image_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_image_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            Some(|c| unsafe { aeron_image_is_closed(c) }),
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_image_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a image."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `image` to get the constants for."]
    #[doc = " \n - `constants` structure to fill in with the constants"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn constants(&self, constants: &AeronImageConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a image."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronImageConstants, AeronCError> {
        let result = AeronImageConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "The position this image has been consumed to by the subscriber."]
    #[doc = ""]
    #[doc = " \n# Return\n the position this image has been consumed to by the subscriber."]
    pub fn position(&self) -> i64 {
        unsafe {
            let result = aeron_image_position(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Set the subscriber position for this image to indicate where it has been consumed to."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `image` to set the position of."]
    pub fn set_position(&self, position: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_set_position(self.get_inner(), position.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Is the current consumed position at the end of the stream?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if at the end of the stream or false if not."]
    pub fn is_end_of_stream(&self) -> bool {
        unsafe {
            let result = aeron_image_is_end_of_stream(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "The position the stream reached when EOS was received from the publisher. The position will be"]
    #[doc = " INT64_MAX until the stream ends and EOS is set."]
    #[doc = ""]
    #[doc = " \n# Return\n position the stream reached when EOS was received from the publisher."]
    pub fn end_of_stream_position(&self) -> i64 {
        unsafe {
            let result = aeron_image_end_of_stream_position(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Count of observed active transports within the image liveness timeout."]
    #[doc = ""]
    #[doc = " If the image is closed, then this is 0. This may also be 0 if no actual datagrams have arrived. IPC"]
    #[doc = " Images also will be 0."]
    #[doc = ""]
    #[doc = " \n# Return\n count of active transports - 0 if Image is closed, no datagrams yet, or IPC. Or -1 for error."]
    pub fn active_transport_count(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_active_transport_count(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Was the associated publication revoked?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if the associated publication was revoked."]
    pub fn is_publication_revoked(&self) -> bool {
        unsafe {
            let result = aeron_image_is_publication_revoked(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified."]
    #[doc = " \n"]
    #[doc = " Use a fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    pub fn poll<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_poll(
                self.get_inner(),
                {
                    let callback: aeron_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified."]
    #[doc = " \n"]
    #[doc = " Use a fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn poll_once<AeronFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> ()>(
        &self,
        mut handler: AeronFragmentHandlerHandlerImpl,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_poll(
                self.get_inner(),
                Some(
                    aeron_fragment_handler_t_callback_for_once_closure::<
                        AeronFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    pub fn controlled_poll<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_controlled_poll(
                self.get_inner(),
                {
                    let callback: aeron_controlled_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_controlled_fragment_handler_t_callback::<
                                AeronControlledFragmentHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn controlled_poll_once<
        AeronControlledFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> aeron_controlled_fragment_handler_action_t,
    >(
        &self,
        mut handler: AeronControlledFragmentHandlerHandlerImpl,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_controlled_poll(
                self.get_inner(),
                Some(
                    aeron_controlled_fragment_handler_t_callback_for_once_closure::<
                        AeronControlledFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified or the maximum position specified."]
    #[doc = " \n"]
    #[doc = " Use a fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` to consume messages up to."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    pub fn bounded_poll<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
        limit_position: i64,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_bounded_poll(
                self.get_inner(),
                {
                    let callback: aeron_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                limit_position.into(),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified or the maximum position specified."]
    #[doc = " \n"]
    #[doc = " Use a fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` to consume messages up to."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn bounded_poll_once<AeronFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> ()>(
        &self,
        mut handler: AeronFragmentHandlerHandlerImpl,
        limit_position: i64,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_bounded_poll(
                self.get_inner(),
                Some(
                    aeron_fragment_handler_t_callback_for_once_closure::<
                        AeronFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                limit_position.into(),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified or the maximum position specified."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` to consume messages up to."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    pub fn bounded_controlled_poll<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
        limit_position: i64,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_bounded_controlled_poll(
                self.get_inner(),
                {
                    let callback: aeron_controlled_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_controlled_fragment_handler_t_callback::<
                                AeronControlledFragmentHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                limit_position.into(),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of fragments as specified or the maximum position specified."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` to consume messages up to."]
    #[doc = " \n - `fragment_limit` for the number of fragments to be consumed during one polling operation."]
    #[doc = " \n# Return\n the number of fragments that have been consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn bounded_controlled_poll_once<
        AeronControlledFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> aeron_controlled_fragment_handler_action_t,
    >(
        &self,
        mut handler: AeronControlledFragmentHandlerHandlerImpl,
        limit_position: i64,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_bounded_controlled_poll(
                self.get_inner(),
                Some(
                    aeron_controlled_fragment_handler_t_callback_for_once_closure::<
                        AeronControlledFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                limit_position.into(),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Peek for new messages in a stream by scanning forward from an initial position. If new messages are found then"]
    #[doc = " they will be delivered to the handler up to a limited position."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments. Scans must also"]
    #[doc = " start at the beginning of a message so that the assembler is reset."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `initial_position` from which to peek forward."]
    #[doc = " \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` up to which can be scanned."]
    #[doc = " \n# Return\n the resulting position after the scan terminates which is a complete message or -1 for error."]
    pub fn controlled_peek<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        &self,
        initial_position: i64,
        handler: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
        limit_position: i64,
    ) -> i64 {
        unsafe {
            let result = aeron_image_controlled_peek(
                self.get_inner(),
                initial_position.into(),
                {
                    let callback: aeron_controlled_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_controlled_fragment_handler_t_callback::<
                                AeronControlledFragmentHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                limit_position.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Peek for new messages in a stream by scanning forward from an initial position. If new messages are found then"]
    #[doc = " they will be delivered to the handler up to a limited position."]
    #[doc = " \n"]
    #[doc = " Use a controlled fragment assembler to assemble messages which span multiple fragments. Scans must also"]
    #[doc = " start at the beginning of a message so that the assembler is reset."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `initial_position` from which to peek forward."]
    #[doc = " \n - `handler` to which message fragments are delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `limit_position` up to which can be scanned."]
    #[doc = " \n# Return\n the resulting position after the scan terminates which is a complete message or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn controlled_peek_once<
        AeronControlledFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> aeron_controlled_fragment_handler_action_t,
    >(
        &self,
        initial_position: i64,
        mut handler: AeronControlledFragmentHandlerHandlerImpl,
        limit_position: i64,
    ) -> i64 {
        unsafe {
            let result = aeron_image_controlled_peek(
                self.get_inner(),
                initial_position.into(),
                Some(
                    aeron_controlled_fragment_handler_t_callback_for_once_closure::<
                        AeronControlledFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                limit_position.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of bytes."]
    #[doc = " \n"]
    #[doc = " A scan will terminate if a padding frame is encountered. If first frame in a scan is padding then a block"]
    #[doc = " for the padding is notified. If the padding comes after the first frame in a scan then the scan terminates"]
    #[doc = " at the offset the padding frame begins. Padding frames are delivered singularly in a block."]
    #[doc = " \n"]
    #[doc = " Padding frames may be for a greater range than the limit offset but only the header needs to be valid so"]
    #[doc = " relevant length of the frame is data header length."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which block is delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `block_length_limit` up to which a block may be in length."]
    #[doc = " \n# Return\n the number of bytes that have been consumed or -1 for error."]
    pub fn block_poll<AeronBlockHandlerHandlerImpl: AeronBlockHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronBlockHandlerHandlerImpl>>,
        block_length_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_block_poll(
                self.get_inner(),
                {
                    let callback: aeron_block_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_block_handler_t_callback::<AeronBlockHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                block_length_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll for new messages in a stream. If new messages are found beyond the last consumed position then they"]
    #[doc = " will be delivered to the handler up to a limited number of bytes."]
    #[doc = " \n"]
    #[doc = " A scan will terminate if a padding frame is encountered. If first frame in a scan is padding then a block"]
    #[doc = " for the padding is notified. If the padding comes after the first frame in a scan then the scan terminates"]
    #[doc = " at the offset the padding frame begins. Padding frames are delivered singularly in a block."]
    #[doc = " \n"]
    #[doc = " Padding frames may be for a greater range than the limit offset but only the header needs to be valid so"]
    #[doc = " relevant length of the frame is data header length."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to which block is delivered."]
    #[doc = " \n - `clientd` to pass to the handler."]
    #[doc = " \n - `block_length_limit` up to which a block may be in length."]
    #[doc = " \n# Return\n the number of bytes that have been consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn block_poll_once<AeronBlockHandlerHandlerImpl: FnMut(&[u8], i32, i32) -> ()>(
        &self,
        mut handler: AeronBlockHandlerHandlerImpl,
        block_length_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_block_poll(
                self.get_inner(),
                Some(
                    aeron_block_handler_t_callback_for_once_closure::<AeronBlockHandlerHandlerImpl>,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                block_length_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_image_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn reject(&self, reason: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_image_reject(self.get_inner(), reason.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_image_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_image_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_image_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronImage {
    type Target = aeron_image_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_image_t> for AeronImage {
    #[inline]
    fn from(value: *mut aeron_image_t) -> Self {
        AeronImage {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronImage> for *mut aeron_image_t {
    #[inline]
    fn from(value: AeronImage) -> Self {
        value.get_inner()
    }
}
impl From<&AeronImage> for *mut aeron_image_t {
    #[inline]
    fn from(value: &AeronImage) -> Self {
        value.get_inner()
    }
}
impl From<AeronImage> for aeron_image_t {
    #[inline]
    fn from(value: AeronImage) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_image_t> for AeronImage {
    #[inline]
    fn from(value: *const aeron_image_t) -> Self {
        AeronImage {
            inner: CResource::Borrowed(value as *mut aeron_image_t),
        }
    }
}
impl From<aeron_image_t> for AeronImage {
    #[inline]
    fn from(value: aeron_image_t) -> Self {
        AeronImage {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronIovec {
    inner: CResource<aeron_iovec_t>,
}
impl core::fmt::Debug for AeronIovec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronIovec))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronIovec))
                .field("inner", &self.inner)
                .field(stringify!(iov_len), &self.iov_len())
                .finish()
        }
    }
}
impl AeronIovec {
    #[inline]
    pub fn new(iov_base: *mut u8, iov_len: usize) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_iovec_t {
                    iov_base: iov_base.into(),
                    iov_len: iov_len.into(),
                };
                let inner_ptr: *mut aeron_iovec_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_iovec_t)
                );
                let inst: aeron_iovec_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_iovec_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_iovec_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn iov_base(&self) -> *mut u8 {
        self.iov_base.into()
    }
    #[inline]
    pub fn iov_len(&self) -> usize {
        self.iov_len.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_iovec_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_iovec_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_iovec_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronIovec {
    type Target = aeron_iovec_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_iovec_t> for AeronIovec {
    #[inline]
    fn from(value: *mut aeron_iovec_t) -> Self {
        AeronIovec {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronIovec> for *mut aeron_iovec_t {
    #[inline]
    fn from(value: AeronIovec) -> Self {
        value.get_inner()
    }
}
impl From<&AeronIovec> for *mut aeron_iovec_t {
    #[inline]
    fn from(value: &AeronIovec) -> Self {
        value.get_inner()
    }
}
impl From<AeronIovec> for aeron_iovec_t {
    #[inline]
    fn from(value: AeronIovec) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_iovec_t> for AeronIovec {
    #[inline]
    fn from(value: *const aeron_iovec_t) -> Self {
        AeronIovec {
            inner: CResource::Borrowed(value as *mut aeron_iovec_t),
        }
    }
}
impl From<aeron_iovec_t> for AeronIovec {
    #[inline]
    fn from(value: aeron_iovec_t) -> Self {
        AeronIovec {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronIovec {
    fn default() -> Self {
        AeronIovec::new_zeroed_on_heap()
    }
}
impl AeronIovec {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronIpcChannelParams {
    inner: CResource<aeron_ipc_channel_params_t>,
}
impl core::fmt::Debug for AeronIpcChannelParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronIpcChannelParams))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronIpcChannelParams))
                .field("inner", &self.inner)
                .field(stringify!(additional_params), &self.additional_params())
                .finish()
        }
    }
}
impl AeronIpcChannelParams {
    #[inline]
    pub fn new(
        channel_tag: &std::ffi::CStr,
        entity_tag: &std::ffi::CStr,
        additional_params: AeronUriParams,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_ipc_channel_params_t {
                    channel_tag: channel_tag.as_ptr(),
                    entity_tag: entity_tag.as_ptr(),
                    additional_params: additional_params.into(),
                };
                let inner_ptr: *mut aeron_ipc_channel_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_ipc_channel_params_t)
                );
                let inst: aeron_ipc_channel_params_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_ipc_channel_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_ipc_channel_params_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn channel_tag(&self) -> &str {
        if self.channel_tag.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.channel_tag).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn entity_tag(&self) -> &str {
        if self.entity_tag.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.entity_tag).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn additional_params(&self) -> AeronUriParams {
        self.additional_params.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_ipc_channel_params_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_ipc_channel_params_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_ipc_channel_params_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronIpcChannelParams {
    type Target = aeron_ipc_channel_params_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_ipc_channel_params_t> for AeronIpcChannelParams {
    #[inline]
    fn from(value: *mut aeron_ipc_channel_params_t) -> Self {
        AeronIpcChannelParams {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronIpcChannelParams> for *mut aeron_ipc_channel_params_t {
    #[inline]
    fn from(value: AeronIpcChannelParams) -> Self {
        value.get_inner()
    }
}
impl From<&AeronIpcChannelParams> for *mut aeron_ipc_channel_params_t {
    #[inline]
    fn from(value: &AeronIpcChannelParams) -> Self {
        value.get_inner()
    }
}
impl From<AeronIpcChannelParams> for aeron_ipc_channel_params_t {
    #[inline]
    fn from(value: AeronIpcChannelParams) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_ipc_channel_params_t> for AeronIpcChannelParams {
    #[inline]
    fn from(value: *const aeron_ipc_channel_params_t) -> Self {
        AeronIpcChannelParams {
            inner: CResource::Borrowed(value as *mut aeron_ipc_channel_params_t),
        }
    }
}
impl From<aeron_ipc_channel_params_t> for AeronIpcChannelParams {
    #[inline]
    fn from(value: aeron_ipc_channel_params_t) -> Self {
        AeronIpcChannelParams {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronIpcChannelParams {
    fn default() -> Self {
        AeronIpcChannelParams::new_zeroed_on_heap()
    }
}
impl AeronIpcChannelParams {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronLogBuffer {
    inner: CResource<aeron_log_buffer_t>,
}
impl core::fmt::Debug for AeronLogBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronLogBuffer))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronLogBuffer))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronLogBuffer {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_log_buffer_t)
                );
                let inst: aeron_log_buffer_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_log_buffer_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_log_buffer_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_log_buffer_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_log_buffer_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_log_buffer_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronLogBuffer {
    type Target = aeron_log_buffer_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_log_buffer_t> for AeronLogBuffer {
    #[inline]
    fn from(value: *mut aeron_log_buffer_t) -> Self {
        AeronLogBuffer {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronLogBuffer> for *mut aeron_log_buffer_t {
    #[inline]
    fn from(value: AeronLogBuffer) -> Self {
        value.get_inner()
    }
}
impl From<&AeronLogBuffer> for *mut aeron_log_buffer_t {
    #[inline]
    fn from(value: &AeronLogBuffer) -> Self {
        value.get_inner()
    }
}
impl From<AeronLogBuffer> for aeron_log_buffer_t {
    #[inline]
    fn from(value: AeronLogBuffer) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_log_buffer_t> for AeronLogBuffer {
    #[inline]
    fn from(value: *const aeron_log_buffer_t) -> Self {
        AeronLogBuffer {
            inner: CResource::Borrowed(value as *mut aeron_log_buffer_t),
        }
    }
}
impl From<aeron_log_buffer_t> for AeronLogBuffer {
    #[inline]
    fn from(value: aeron_log_buffer_t) -> Self {
        AeronLogBuffer {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronLogbufferMetadata {
    inner: CResource<aeron_logbuffer_metadata_t>,
}
impl core::fmt::Debug for AeronLogbufferMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronLogbufferMetadata))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronLogbufferMetadata))
                .field("inner", &self.inner)
                .field(stringify!(active_term_count), &self.active_term_count())
                .field(
                    stringify!(end_of_stream_position),
                    &self.end_of_stream_position(),
                )
                .field(stringify!(is_connected), &self.is_connected())
                .field(
                    stringify!(active_transport_count),
                    &self.active_transport_count(),
                )
                .field(stringify!(correlation_id), &self.correlation_id())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(
                    stringify!(default_frame_header_length),
                    &self.default_frame_header_length(),
                )
                .field(stringify!(mtu_length), &self.mtu_length())
                .field(stringify!(term_length), &self.term_length())
                .field(stringify!(page_size), &self.page_size())
                .field(
                    stringify!(publication_window_length),
                    &self.publication_window_length(),
                )
                .field(
                    stringify!(receiver_window_length),
                    &self.receiver_window_length(),
                )
                .field(
                    stringify!(socket_sndbuf_length),
                    &self.socket_sndbuf_length(),
                )
                .field(
                    stringify!(os_default_socket_sndbuf_length),
                    &self.os_default_socket_sndbuf_length(),
                )
                .field(
                    stringify!(os_max_socket_sndbuf_length),
                    &self.os_max_socket_sndbuf_length(),
                )
                .field(
                    stringify!(socket_rcvbuf_length),
                    &self.socket_rcvbuf_length(),
                )
                .field(
                    stringify!(os_default_socket_rcvbuf_length),
                    &self.os_default_socket_rcvbuf_length(),
                )
                .field(
                    stringify!(os_max_socket_rcvbuf_length),
                    &self.os_max_socket_rcvbuf_length(),
                )
                .field(stringify!(max_resend), &self.max_resend())
                .field(stringify!(entity_tag), &self.entity_tag())
                .field(
                    stringify!(response_correlation_id),
                    &self.response_correlation_id(),
                )
                .field(stringify!(linger_timeout_ns), &self.linger_timeout_ns())
                .field(
                    stringify!(untethered_window_limit_timeout_ns),
                    &self.untethered_window_limit_timeout_ns(),
                )
                .field(
                    stringify!(untethered_resting_timeout_ns),
                    &self.untethered_resting_timeout_ns(),
                )
                .field(
                    stringify!(untethered_linger_timeout_ns),
                    &self.untethered_linger_timeout_ns(),
                )
                .finish()
        }
    }
}
impl AeronLogbufferMetadata {
    #[inline]
    pub fn new(
        term_tail_counters: [i64; 3usize],
        active_term_count: i32,
        pad1: [u8; 100usize],
        end_of_stream_position: i64,
        is_connected: i32,
        active_transport_count: i32,
        pad2: [u8; 112usize],
        correlation_id: i64,
        initial_term_id: i32,
        default_frame_header_length: i32,
        mtu_length: i32,
        term_length: i32,
        page_size: i32,
        publication_window_length: i32,
        receiver_window_length: i32,
        socket_sndbuf_length: i32,
        os_default_socket_sndbuf_length: i32,
        os_max_socket_sndbuf_length: i32,
        socket_rcvbuf_length: i32,
        os_default_socket_rcvbuf_length: i32,
        os_max_socket_rcvbuf_length: i32,
        max_resend: i32,
        default_header: [u8; 128usize],
        entity_tag: i64,
        response_correlation_id: i64,
        linger_timeout_ns: i64,
        untethered_window_limit_timeout_ns: i64,
        untethered_resting_timeout_ns: i64,
        group: u8,
        is_response: u8,
        rejoin: u8,
        reliable: u8,
        sparse: u8,
        signal_eos: u8,
        spies_simulate_connection: u8,
        tether: u8,
        is_publication_revoked: u8,
        pad3: [u8; 3usize],
        untethered_linger_timeout_ns: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_logbuffer_metadata_t {
                    term_tail_counters: term_tail_counters.into(),
                    active_term_count: active_term_count.into(),
                    pad1: pad1.into(),
                    end_of_stream_position: end_of_stream_position.into(),
                    is_connected: is_connected.into(),
                    active_transport_count: active_transport_count.into(),
                    pad2: pad2.into(),
                    correlation_id: correlation_id.into(),
                    initial_term_id: initial_term_id.into(),
                    default_frame_header_length: default_frame_header_length.into(),
                    mtu_length: mtu_length.into(),
                    term_length: term_length.into(),
                    page_size: page_size.into(),
                    publication_window_length: publication_window_length.into(),
                    receiver_window_length: receiver_window_length.into(),
                    socket_sndbuf_length: socket_sndbuf_length.into(),
                    os_default_socket_sndbuf_length: os_default_socket_sndbuf_length.into(),
                    os_max_socket_sndbuf_length: os_max_socket_sndbuf_length.into(),
                    socket_rcvbuf_length: socket_rcvbuf_length.into(),
                    os_default_socket_rcvbuf_length: os_default_socket_rcvbuf_length.into(),
                    os_max_socket_rcvbuf_length: os_max_socket_rcvbuf_length.into(),
                    max_resend: max_resend.into(),
                    default_header: default_header.into(),
                    entity_tag: entity_tag.into(),
                    response_correlation_id: response_correlation_id.into(),
                    linger_timeout_ns: linger_timeout_ns.into(),
                    untethered_window_limit_timeout_ns: untethered_window_limit_timeout_ns.into(),
                    untethered_resting_timeout_ns: untethered_resting_timeout_ns.into(),
                    group: group.into(),
                    is_response: is_response.into(),
                    rejoin: rejoin.into(),
                    reliable: reliable.into(),
                    sparse: sparse.into(),
                    signal_eos: signal_eos.into(),
                    spies_simulate_connection: spies_simulate_connection.into(),
                    tether: tether.into(),
                    is_publication_revoked: is_publication_revoked.into(),
                    pad3: pad3.into(),
                    untethered_linger_timeout_ns: untethered_linger_timeout_ns.into(),
                };
                let inner_ptr: *mut aeron_logbuffer_metadata_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_logbuffer_metadata_t)
                );
                let inst: aeron_logbuffer_metadata_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_logbuffer_metadata_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_logbuffer_metadata_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn term_tail_counters(&self) -> [i64; 3usize] {
        self.term_tail_counters.into()
    }
    #[inline]
    pub fn active_term_count(&self) -> i32 {
        self.active_term_count.into()
    }
    #[inline]
    pub fn pad1(&self) -> [u8; 100usize] {
        self.pad1.into()
    }
    #[inline]
    pub fn end_of_stream_position(&self) -> i64 {
        self.end_of_stream_position.into()
    }
    #[inline]
    pub fn is_connected(&self) -> i32 {
        self.is_connected.into()
    }
    #[inline]
    pub fn active_transport_count(&self) -> i32 {
        self.active_transport_count.into()
    }
    #[inline]
    pub fn pad2(&self) -> [u8; 112usize] {
        self.pad2.into()
    }
    #[inline]
    pub fn correlation_id(&self) -> i64 {
        self.correlation_id.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn default_frame_header_length(&self) -> i32 {
        self.default_frame_header_length.into()
    }
    #[inline]
    pub fn mtu_length(&self) -> i32 {
        self.mtu_length.into()
    }
    #[inline]
    pub fn term_length(&self) -> i32 {
        self.term_length.into()
    }
    #[inline]
    pub fn page_size(&self) -> i32 {
        self.page_size.into()
    }
    #[inline]
    pub fn publication_window_length(&self) -> i32 {
        self.publication_window_length.into()
    }
    #[inline]
    pub fn receiver_window_length(&self) -> i32 {
        self.receiver_window_length.into()
    }
    #[inline]
    pub fn socket_sndbuf_length(&self) -> i32 {
        self.socket_sndbuf_length.into()
    }
    #[inline]
    pub fn os_default_socket_sndbuf_length(&self) -> i32 {
        self.os_default_socket_sndbuf_length.into()
    }
    #[inline]
    pub fn os_max_socket_sndbuf_length(&self) -> i32 {
        self.os_max_socket_sndbuf_length.into()
    }
    #[inline]
    pub fn socket_rcvbuf_length(&self) -> i32 {
        self.socket_rcvbuf_length.into()
    }
    #[inline]
    pub fn os_default_socket_rcvbuf_length(&self) -> i32 {
        self.os_default_socket_rcvbuf_length.into()
    }
    #[inline]
    pub fn os_max_socket_rcvbuf_length(&self) -> i32 {
        self.os_max_socket_rcvbuf_length.into()
    }
    #[inline]
    pub fn max_resend(&self) -> i32 {
        self.max_resend.into()
    }
    #[inline]
    pub fn default_header(&self) -> [u8; 128usize] {
        self.default_header.into()
    }
    #[inline]
    pub fn entity_tag(&self) -> i64 {
        self.entity_tag.into()
    }
    #[inline]
    pub fn response_correlation_id(&self) -> i64 {
        self.response_correlation_id.into()
    }
    #[inline]
    pub fn linger_timeout_ns(&self) -> i64 {
        self.linger_timeout_ns.into()
    }
    #[inline]
    pub fn untethered_window_limit_timeout_ns(&self) -> i64 {
        self.untethered_window_limit_timeout_ns.into()
    }
    #[inline]
    pub fn untethered_resting_timeout_ns(&self) -> i64 {
        self.untethered_resting_timeout_ns.into()
    }
    #[inline]
    pub fn group(&self) -> u8 {
        self.group.into()
    }
    #[inline]
    pub fn is_response(&self) -> u8 {
        self.is_response.into()
    }
    #[inline]
    pub fn rejoin(&self) -> u8 {
        self.rejoin.into()
    }
    #[inline]
    pub fn reliable(&self) -> u8 {
        self.reliable.into()
    }
    #[inline]
    pub fn sparse(&self) -> u8 {
        self.sparse.into()
    }
    #[inline]
    pub fn signal_eos(&self) -> u8 {
        self.signal_eos.into()
    }
    #[inline]
    pub fn spies_simulate_connection(&self) -> u8 {
        self.spies_simulate_connection.into()
    }
    #[inline]
    pub fn tether(&self) -> u8 {
        self.tether.into()
    }
    #[inline]
    pub fn is_publication_revoked(&self) -> u8 {
        self.is_publication_revoked.into()
    }
    #[inline]
    pub fn pad3(&self) -> [u8; 3usize] {
        self.pad3.into()
    }
    #[inline]
    pub fn untethered_linger_timeout_ns(&self) -> i64 {
        self.untethered_linger_timeout_ns.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_logbuffer_metadata_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_logbuffer_metadata_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_logbuffer_metadata_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronLogbufferMetadata {
    type Target = aeron_logbuffer_metadata_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_logbuffer_metadata_t> for AeronLogbufferMetadata {
    #[inline]
    fn from(value: *mut aeron_logbuffer_metadata_t) -> Self {
        AeronLogbufferMetadata {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronLogbufferMetadata> for *mut aeron_logbuffer_metadata_t {
    #[inline]
    fn from(value: AeronLogbufferMetadata) -> Self {
        value.get_inner()
    }
}
impl From<&AeronLogbufferMetadata> for *mut aeron_logbuffer_metadata_t {
    #[inline]
    fn from(value: &AeronLogbufferMetadata) -> Self {
        value.get_inner()
    }
}
impl From<AeronLogbufferMetadata> for aeron_logbuffer_metadata_t {
    #[inline]
    fn from(value: AeronLogbufferMetadata) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_logbuffer_metadata_t> for AeronLogbufferMetadata {
    #[inline]
    fn from(value: *const aeron_logbuffer_metadata_t) -> Self {
        AeronLogbufferMetadata {
            inner: CResource::Borrowed(value as *mut aeron_logbuffer_metadata_t),
        }
    }
}
impl From<aeron_logbuffer_metadata_t> for AeronLogbufferMetadata {
    #[inline]
    fn from(value: aeron_logbuffer_metadata_t) -> Self {
        AeronLogbufferMetadata {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronLogbufferMetadata {
    fn default() -> Self {
        AeronLogbufferMetadata::new_zeroed_on_heap()
    }
}
impl AeronLogbufferMetadata {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronLossReporterEntry {
    inner: CResource<aeron_loss_reporter_entry_t>,
}
impl core::fmt::Debug for AeronLossReporterEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronLossReporterEntry))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronLossReporterEntry))
                .field("inner", &self.inner)
                .field(stringify!(observation_count), &self.observation_count())
                .field(stringify!(total_bytes_lost), &self.total_bytes_lost())
                .field(
                    stringify!(first_observation_timestamp),
                    &self.first_observation_timestamp(),
                )
                .field(
                    stringify!(last_observation_timestamp),
                    &self.last_observation_timestamp(),
                )
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .finish()
        }
    }
}
impl AeronLossReporterEntry {
    #[inline]
    pub fn new(
        observation_count: i64,
        total_bytes_lost: i64,
        first_observation_timestamp: i64,
        last_observation_timestamp: i64,
        session_id: i32,
        stream_id: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_loss_reporter_entry_t {
                    observation_count: observation_count.into(),
                    total_bytes_lost: total_bytes_lost.into(),
                    first_observation_timestamp: first_observation_timestamp.into(),
                    last_observation_timestamp: last_observation_timestamp.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                };
                let inner_ptr: *mut aeron_loss_reporter_entry_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_loss_reporter_entry_t)
                );
                let inst: aeron_loss_reporter_entry_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_loss_reporter_entry_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_loss_reporter_entry_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn observation_count(&self) -> i64 {
        self.observation_count.into()
    }
    #[inline]
    pub fn total_bytes_lost(&self) -> i64 {
        self.total_bytes_lost.into()
    }
    #[inline]
    pub fn first_observation_timestamp(&self) -> i64 {
        self.first_observation_timestamp.into()
    }
    #[inline]
    pub fn last_observation_timestamp(&self) -> i64 {
        self.last_observation_timestamp.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_loss_reporter_entry_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_loss_reporter_entry_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_loss_reporter_entry_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronLossReporterEntry {
    type Target = aeron_loss_reporter_entry_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_loss_reporter_entry_t> for AeronLossReporterEntry {
    #[inline]
    fn from(value: *mut aeron_loss_reporter_entry_t) -> Self {
        AeronLossReporterEntry {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronLossReporterEntry> for *mut aeron_loss_reporter_entry_t {
    #[inline]
    fn from(value: AeronLossReporterEntry) -> Self {
        value.get_inner()
    }
}
impl From<&AeronLossReporterEntry> for *mut aeron_loss_reporter_entry_t {
    #[inline]
    fn from(value: &AeronLossReporterEntry) -> Self {
        value.get_inner()
    }
}
impl From<AeronLossReporterEntry> for aeron_loss_reporter_entry_t {
    #[inline]
    fn from(value: AeronLossReporterEntry) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_loss_reporter_entry_t> for AeronLossReporterEntry {
    #[inline]
    fn from(value: *const aeron_loss_reporter_entry_t) -> Self {
        AeronLossReporterEntry {
            inner: CResource::Borrowed(value as *mut aeron_loss_reporter_entry_t),
        }
    }
}
impl From<aeron_loss_reporter_entry_t> for AeronLossReporterEntry {
    #[inline]
    fn from(value: aeron_loss_reporter_entry_t) -> Self {
        AeronLossReporterEntry {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronLossReporterEntry {
    fn default() -> Self {
        AeronLossReporterEntry::new_zeroed_on_heap()
    }
}
impl AeronLossReporterEntry {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronLossReporter {
    inner: CResource<aeron_loss_reporter_t>,
}
impl core::fmt::Debug for AeronLossReporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronLossReporter))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronLossReporter))
                .field("inner", &self.inner)
                .field(stringify!(next_record_offset), &self.next_record_offset())
                .field(stringify!(capacity), &self.capacity())
                .finish()
        }
    }
}
impl AeronLossReporter {
    #[inline]
    pub fn new(
        buffer: *mut u8,
        next_record_offset: usize,
        capacity: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_loss_reporter_t {
                    buffer: buffer.into(),
                    next_record_offset: next_record_offset.into(),
                    capacity: capacity.into(),
                };
                let inner_ptr: *mut aeron_loss_reporter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_loss_reporter_t)
                );
                let inst: aeron_loss_reporter_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_loss_reporter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_loss_reporter_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn buffer(&self) -> *mut u8 {
        self.buffer.into()
    }
    #[inline]
    pub fn next_record_offset(&self) -> usize {
        self.next_record_offset.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn init(&self, buffer: &mut [u8]) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_loss_reporter_init(self.get_inner(), buffer.as_ptr() as *mut _, buffer.len());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn create_entry(
        &self,
        initial_bytes_lost: i64,
        timestamp_ms: i64,
        session_id: i32,
        stream_id: i32,
        channel: &str,
        source: &str,
    ) -> aeron_loss_reporter_entry_offset_t {
        unsafe {
            let result = aeron_loss_reporter_create_entry(
                self.get_inner(),
                initial_bytes_lost.into(),
                timestamp_ms.into(),
                session_id.into(),
                stream_id.into(),
                channel.as_ptr() as *const _,
                channel.len(),
                source.as_ptr() as *const _,
                source.len(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn record_observation(
        &self,
        offset: aeron_loss_reporter_entry_offset_t,
        bytes_lost: i64,
        timestamp_ms: i64,
    ) -> () {
        unsafe {
            let result = aeron_loss_reporter_record_observation(
                self.get_inner(),
                offset.into(),
                bytes_lost.into(),
                timestamp_ms.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn resolve_filename(
        directory: &std::ffi::CStr,
        filename_buffer: *mut ::std::os::raw::c_char,
        filename_buffer_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_loss_reporter_resolve_filename(
                directory.as_ptr(),
                filename_buffer.into(),
                filename_buffer_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn read<
        AeronLossReporterReadEntryFuncHandlerImpl: AeronLossReporterReadEntryFuncCallback,
    >(
        buffer: *const u8,
        capacity: usize,
        entry_func: Option<&Handler<AeronLossReporterReadEntryFuncHandlerImpl>>,
    ) -> usize {
        unsafe {
            let result = aeron_loss_reporter_read(
                buffer.into(),
                capacity.into(),
                {
                    let callback: aeron_loss_reporter_read_entry_func_t = if entry_func.is_none() {
                        None
                    } else {
                        Some(
                            aeron_loss_reporter_read_entry_func_t_callback::<
                                AeronLossReporterReadEntryFuncHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                entry_func
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn read_once<
        AeronLossReporterReadEntryFuncHandlerImpl: FnMut(i64, i64, i64, i64, i32, i32, &str, &str) -> (),
    >(
        buffer: *const u8,
        capacity: usize,
        mut entry_func: AeronLossReporterReadEntryFuncHandlerImpl,
    ) -> usize {
        unsafe {
            let result = aeron_loss_reporter_read(
                buffer.into(),
                capacity.into(),
                Some(
                    aeron_loss_reporter_read_entry_func_t_callback_for_once_closure::<
                        AeronLossReporterReadEntryFuncHandlerImpl,
                    >,
                ),
                &mut entry_func as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_loss_reporter_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_loss_reporter_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_loss_reporter_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronLossReporter {
    type Target = aeron_loss_reporter_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_loss_reporter_t> for AeronLossReporter {
    #[inline]
    fn from(value: *mut aeron_loss_reporter_t) -> Self {
        AeronLossReporter {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronLossReporter> for *mut aeron_loss_reporter_t {
    #[inline]
    fn from(value: AeronLossReporter) -> Self {
        value.get_inner()
    }
}
impl From<&AeronLossReporter> for *mut aeron_loss_reporter_t {
    #[inline]
    fn from(value: &AeronLossReporter) -> Self {
        value.get_inner()
    }
}
impl From<AeronLossReporter> for aeron_loss_reporter_t {
    #[inline]
    fn from(value: AeronLossReporter) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_loss_reporter_t> for AeronLossReporter {
    #[inline]
    fn from(value: *const aeron_loss_reporter_t) -> Self {
        AeronLossReporter {
            inner: CResource::Borrowed(value as *mut aeron_loss_reporter_t),
        }
    }
}
impl From<aeron_loss_reporter_t> for AeronLossReporter {
    #[inline]
    fn from(value: aeron_loss_reporter_t) -> Self {
        AeronLossReporter {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronLossReporter {
    fn default() -> Self {
        AeronLossReporter::new_zeroed_on_heap()
    }
}
impl AeronLossReporter {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronMappedBuffer {
    inner: CResource<aeron_mapped_buffer_t>,
}
impl core::fmt::Debug for AeronMappedBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronMappedBuffer))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronMappedBuffer))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronMappedBuffer {
    #[inline]
    pub fn new(addr: &mut [u8]) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_mapped_buffer_t {
                    addr: addr.as_ptr() as *mut _,
                    length: addr.len(),
                };
                let inner_ptr: *mut aeron_mapped_buffer_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_mapped_buffer_t)
                );
                let inst: aeron_mapped_buffer_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_mapped_buffer_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_mapped_buffer_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn addr(&self) -> &mut [u8] {
        unsafe {
            if self.addr.is_null() {
                &mut [] as &mut [_]
            } else {
                std::slice::from_raw_parts_mut(self.addr, self.length.try_into().unwrap())
            }
        }
    }
    #[inline]
    pub fn length(&self) -> usize {
        self.length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_mapped_buffer_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_mapped_buffer_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_mapped_buffer_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronMappedBuffer {
    type Target = aeron_mapped_buffer_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_mapped_buffer_t> for AeronMappedBuffer {
    #[inline]
    fn from(value: *mut aeron_mapped_buffer_t) -> Self {
        AeronMappedBuffer {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronMappedBuffer> for *mut aeron_mapped_buffer_t {
    #[inline]
    fn from(value: AeronMappedBuffer) -> Self {
        value.get_inner()
    }
}
impl From<&AeronMappedBuffer> for *mut aeron_mapped_buffer_t {
    #[inline]
    fn from(value: &AeronMappedBuffer) -> Self {
        value.get_inner()
    }
}
impl From<AeronMappedBuffer> for aeron_mapped_buffer_t {
    #[inline]
    fn from(value: AeronMappedBuffer) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_mapped_buffer_t> for AeronMappedBuffer {
    #[inline]
    fn from(value: *const aeron_mapped_buffer_t) -> Self {
        AeronMappedBuffer {
            inner: CResource::Borrowed(value as *mut aeron_mapped_buffer_t),
        }
    }
}
impl From<aeron_mapped_buffer_t> for AeronMappedBuffer {
    #[inline]
    fn from(value: aeron_mapped_buffer_t) -> Self {
        AeronMappedBuffer {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronMappedBuffer {
    fn default() -> Self {
        AeronMappedBuffer::new_zeroed_on_heap()
    }
}
impl AeronMappedBuffer {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronMappedFile {
    inner: CResource<aeron_mapped_file_t>,
}
impl core::fmt::Debug for AeronMappedFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronMappedFile))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronMappedFile))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronMappedFile {
    #[inline]
    pub fn new(addr: *mut ::std::os::raw::c_void, length: usize) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_mapped_file_t {
                    addr: addr.into(),
                    length: length.into(),
                };
                let inner_ptr: *mut aeron_mapped_file_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_mapped_file_t)
                );
                let inst: aeron_mapped_file_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_mapped_file_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_mapped_file_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn addr(&self) -> *mut ::std::os::raw::c_void {
        self.addr.into()
    }
    #[inline]
    pub fn length(&self) -> usize {
        self.length.into()
    }
    #[inline]
    pub fn aeron_map_new_file(
        &self,
        path: &std::ffi::CStr,
        fill_with_zeroes: bool,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_map_new_file(self.get_inner(), path.as_ptr(), fill_with_zeroes.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_map_existing_file(&self, path: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_map_existing_file(self.get_inner(), path.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_unmap(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_unmap(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_mapped_file_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_mapped_file_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_mapped_file_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronMappedFile {
    type Target = aeron_mapped_file_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_mapped_file_t> for AeronMappedFile {
    #[inline]
    fn from(value: *mut aeron_mapped_file_t) -> Self {
        AeronMappedFile {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronMappedFile> for *mut aeron_mapped_file_t {
    #[inline]
    fn from(value: AeronMappedFile) -> Self {
        value.get_inner()
    }
}
impl From<&AeronMappedFile> for *mut aeron_mapped_file_t {
    #[inline]
    fn from(value: &AeronMappedFile) -> Self {
        value.get_inner()
    }
}
impl From<AeronMappedFile> for aeron_mapped_file_t {
    #[inline]
    fn from(value: AeronMappedFile) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_mapped_file_t> for AeronMappedFile {
    #[inline]
    fn from(value: *const aeron_mapped_file_t) -> Self {
        AeronMappedFile {
            inner: CResource::Borrowed(value as *mut aeron_mapped_file_t),
        }
    }
}
impl From<aeron_mapped_file_t> for AeronMappedFile {
    #[inline]
    fn from(value: aeron_mapped_file_t) -> Self {
        AeronMappedFile {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronMappedFile {
    fn default() -> Self {
        AeronMappedFile::new_zeroed_on_heap()
    }
}
impl AeronMappedFile {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronMappedRawLog {
    inner: CResource<aeron_mapped_raw_log_t>,
}
impl core::fmt::Debug for AeronMappedRawLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronMappedRawLog))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronMappedRawLog))
                .field("inner", &self.inner)
                .field(stringify!(log_meta_data), &self.log_meta_data())
                .field(stringify!(mapped_file), &self.mapped_file())
                .field(stringify!(term_length), &self.term_length())
                .finish()
        }
    }
}
impl AeronMappedRawLog {
    #[inline]
    pub fn new(
        term_buffers: [aeron_mapped_buffer_t; 3usize],
        log_meta_data: AeronMappedBuffer,
        mapped_file: AeronMappedFile,
        term_length: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_mapped_raw_log_t {
                    term_buffers: term_buffers.into(),
                    log_meta_data: log_meta_data.into(),
                    mapped_file: mapped_file.into(),
                    term_length: term_length.into(),
                };
                let inner_ptr: *mut aeron_mapped_raw_log_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_mapped_raw_log_t)
                );
                let inst: aeron_mapped_raw_log_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_mapped_raw_log_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_mapped_raw_log_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn term_buffers(&self) -> [aeron_mapped_buffer_t; 3usize] {
        self.term_buffers.into()
    }
    #[inline]
    pub fn log_meta_data(&self) -> AeronMappedBuffer {
        self.log_meta_data.into()
    }
    #[inline]
    pub fn mapped_file(&self) -> AeronMappedFile {
        self.mapped_file.into()
    }
    #[inline]
    pub fn term_length(&self) -> usize {
        self.term_length.into()
    }
    #[inline]
    pub fn aeron_raw_log_map(
        &self,
        path: &std::ffi::CStr,
        use_sparse_files: bool,
        term_length: u64,
        page_size: u64,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_raw_log_map(
                self.get_inner(),
                path.as_ptr(),
                use_sparse_files.into(),
                term_length.into(),
                page_size.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_raw_log_map_existing(
        &self,
        path: &std::ffi::CStr,
        pre_touch: bool,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_raw_log_map_existing(self.get_inner(), path.as_ptr(), pre_touch.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_raw_log_close(&self, filename: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_raw_log_close(self.get_inner(), filename.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_raw_log_free(&self, filename: &std::ffi::CStr) -> bool {
        unsafe {
            let result = aeron_raw_log_free(self.get_inner(), filename.as_ptr());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_mapped_raw_log_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_mapped_raw_log_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_mapped_raw_log_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronMappedRawLog {
    type Target = aeron_mapped_raw_log_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_mapped_raw_log_t> for AeronMappedRawLog {
    #[inline]
    fn from(value: *mut aeron_mapped_raw_log_t) -> Self {
        AeronMappedRawLog {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronMappedRawLog> for *mut aeron_mapped_raw_log_t {
    #[inline]
    fn from(value: AeronMappedRawLog) -> Self {
        value.get_inner()
    }
}
impl From<&AeronMappedRawLog> for *mut aeron_mapped_raw_log_t {
    #[inline]
    fn from(value: &AeronMappedRawLog) -> Self {
        value.get_inner()
    }
}
impl From<AeronMappedRawLog> for aeron_mapped_raw_log_t {
    #[inline]
    fn from(value: AeronMappedRawLog) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_mapped_raw_log_t> for AeronMappedRawLog {
    #[inline]
    fn from(value: *const aeron_mapped_raw_log_t) -> Self {
        AeronMappedRawLog {
            inner: CResource::Borrowed(value as *mut aeron_mapped_raw_log_t),
        }
    }
}
impl From<aeron_mapped_raw_log_t> for AeronMappedRawLog {
    #[inline]
    fn from(value: aeron_mapped_raw_log_t) -> Self {
        AeronMappedRawLog {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronMappedRawLog {
    fn default() -> Self {
        AeronMappedRawLog::new_zeroed_on_heap()
    }
}
impl AeronMappedRawLog {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronNakHeader {
    inner: CResource<aeron_nak_header_t>,
}
impl core::fmt::Debug for AeronNakHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronNakHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronNakHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(term_id), &self.term_id())
                .field(stringify!(term_offset), &self.term_offset())
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronNakHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        session_id: i32,
        stream_id: i32,
        term_id: i32,
        term_offset: i32,
        length: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_nak_header_t {
                    frame_header: frame_header.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    term_id: term_id.into(),
                    term_offset: term_offset.into(),
                    length: length.into(),
                };
                let inner_ptr: *mut aeron_nak_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_nak_header_t)
                );
                let inst: aeron_nak_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_nak_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_nak_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn term_id(&self) -> i32 {
        self.term_id.into()
    }
    #[inline]
    pub fn term_offset(&self) -> i32 {
        self.term_offset.into()
    }
    #[inline]
    pub fn length(&self) -> i32 {
        self.length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_nak_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_nak_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_nak_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronNakHeader {
    type Target = aeron_nak_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_nak_header_t> for AeronNakHeader {
    #[inline]
    fn from(value: *mut aeron_nak_header_t) -> Self {
        AeronNakHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronNakHeader> for *mut aeron_nak_header_t {
    #[inline]
    fn from(value: AeronNakHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronNakHeader> for *mut aeron_nak_header_t {
    #[inline]
    fn from(value: &AeronNakHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronNakHeader> for aeron_nak_header_t {
    #[inline]
    fn from(value: AeronNakHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_nak_header_t> for AeronNakHeader {
    #[inline]
    fn from(value: *const aeron_nak_header_t) -> Self {
        AeronNakHeader {
            inner: CResource::Borrowed(value as *mut aeron_nak_header_t),
        }
    }
}
impl From<aeron_nak_header_t> for AeronNakHeader {
    #[inline]
    fn from(value: aeron_nak_header_t) -> Self {
        AeronNakHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronNakHeader {
    fn default() -> Self {
        AeronNakHeader::new_zeroed_on_heap()
    }
}
impl AeronNakHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronAvailableCounterPair {
    inner: CResource<aeron_on_available_counter_pair_t>,
}
impl core::fmt::Debug for AeronAvailableCounterPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronAvailableCounterPair))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronAvailableCounterPair))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronAvailableCounterPair {
    #[inline]
    pub fn new<AeronAvailableCounterHandlerImpl: AeronAvailableCounterCallback>(
        handler: Option<&Handler<AeronAvailableCounterHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_on_available_counter_pair_t {
                    handler: {
                        let callback: aeron_on_available_counter_t = if handler.is_none() {
                            None
                        } else {
                            Some(
                                aeron_on_available_counter_t_callback::<
                                    AeronAvailableCounterHandlerImpl,
                                >,
                            )
                        };
                        callback
                    },
                    clientd: handler
                        .map(|m| m.as_raw())
                        .unwrap_or_else(|| std::ptr::null_mut()),
                };
                let inner_ptr: *mut aeron_on_available_counter_pair_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_on_available_counter_pair_t)
                );
                let inst: aeron_on_available_counter_pair_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_on_available_counter_pair_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_on_available_counter_pair_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn handler(&self) -> aeron_on_available_counter_t {
        self.handler.into()
    }
    #[inline]
    pub fn clientd(&self) -> *mut ::std::os::raw::c_void {
        self.clientd.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_on_available_counter_pair_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_on_available_counter_pair_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_on_available_counter_pair_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronAvailableCounterPair {
    type Target = aeron_on_available_counter_pair_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_on_available_counter_pair_t> for AeronAvailableCounterPair {
    #[inline]
    fn from(value: *mut aeron_on_available_counter_pair_t) -> Self {
        AeronAvailableCounterPair {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronAvailableCounterPair> for *mut aeron_on_available_counter_pair_t {
    #[inline]
    fn from(value: AeronAvailableCounterPair) -> Self {
        value.get_inner()
    }
}
impl From<&AeronAvailableCounterPair> for *mut aeron_on_available_counter_pair_t {
    #[inline]
    fn from(value: &AeronAvailableCounterPair) -> Self {
        value.get_inner()
    }
}
impl From<AeronAvailableCounterPair> for aeron_on_available_counter_pair_t {
    #[inline]
    fn from(value: AeronAvailableCounterPair) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_on_available_counter_pair_t> for AeronAvailableCounterPair {
    #[inline]
    fn from(value: *const aeron_on_available_counter_pair_t) -> Self {
        AeronAvailableCounterPair {
            inner: CResource::Borrowed(value as *mut aeron_on_available_counter_pair_t),
        }
    }
}
impl From<aeron_on_available_counter_pair_t> for AeronAvailableCounterPair {
    #[inline]
    fn from(value: aeron_on_available_counter_pair_t) -> Self {
        AeronAvailableCounterPair {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronAvailableCounterPair {
    fn default() -> Self {
        AeronAvailableCounterPair::new_zeroed_on_heap()
    }
}
impl AeronAvailableCounterPair {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronCloseClientPair {
    inner: CResource<aeron_on_close_client_pair_t>,
}
impl core::fmt::Debug for AeronCloseClientPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronCloseClientPair))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronCloseClientPair))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronCloseClientPair {
    #[inline]
    pub fn new<AeronCloseClientHandlerImpl: AeronCloseClientCallback>(
        handler: Option<&Handler<AeronCloseClientHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_on_close_client_pair_t {
                    handler: {
                        let callback: aeron_on_close_client_t = if handler.is_none() {
                            None
                        } else {
                            Some(aeron_on_close_client_t_callback::<AeronCloseClientHandlerImpl>)
                        };
                        callback
                    },
                    clientd: handler
                        .map(|m| m.as_raw())
                        .unwrap_or_else(|| std::ptr::null_mut()),
                };
                let inner_ptr: *mut aeron_on_close_client_pair_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_on_close_client_pair_t)
                );
                let inst: aeron_on_close_client_pair_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_on_close_client_pair_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_on_close_client_pair_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn handler(&self) -> aeron_on_close_client_t {
        self.handler.into()
    }
    #[inline]
    pub fn clientd(&self) -> *mut ::std::os::raw::c_void {
        self.clientd.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_on_close_client_pair_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_on_close_client_pair_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_on_close_client_pair_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronCloseClientPair {
    type Target = aeron_on_close_client_pair_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_on_close_client_pair_t> for AeronCloseClientPair {
    #[inline]
    fn from(value: *mut aeron_on_close_client_pair_t) -> Self {
        AeronCloseClientPair {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronCloseClientPair> for *mut aeron_on_close_client_pair_t {
    #[inline]
    fn from(value: AeronCloseClientPair) -> Self {
        value.get_inner()
    }
}
impl From<&AeronCloseClientPair> for *mut aeron_on_close_client_pair_t {
    #[inline]
    fn from(value: &AeronCloseClientPair) -> Self {
        value.get_inner()
    }
}
impl From<AeronCloseClientPair> for aeron_on_close_client_pair_t {
    #[inline]
    fn from(value: AeronCloseClientPair) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_on_close_client_pair_t> for AeronCloseClientPair {
    #[inline]
    fn from(value: *const aeron_on_close_client_pair_t) -> Self {
        AeronCloseClientPair {
            inner: CResource::Borrowed(value as *mut aeron_on_close_client_pair_t),
        }
    }
}
impl From<aeron_on_close_client_pair_t> for AeronCloseClientPair {
    #[inline]
    fn from(value: aeron_on_close_client_pair_t) -> Self {
        AeronCloseClientPair {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronCloseClientPair {
    fn default() -> Self {
        AeronCloseClientPair::new_zeroed_on_heap()
    }
}
impl AeronCloseClientPair {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronUnavailableCounterPair {
    inner: CResource<aeron_on_unavailable_counter_pair_t>,
}
impl core::fmt::Debug for AeronUnavailableCounterPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUnavailableCounterPair))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUnavailableCounterPair))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronUnavailableCounterPair {
    #[inline]
    pub fn new<AeronUnavailableCounterHandlerImpl: AeronUnavailableCounterCallback>(
        handler: Option<&Handler<AeronUnavailableCounterHandlerImpl>>,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_on_unavailable_counter_pair_t {
                    handler: {
                        let callback: aeron_on_unavailable_counter_t = if handler.is_none() {
                            None
                        } else {
                            Some(
                                aeron_on_unavailable_counter_t_callback::<
                                    AeronUnavailableCounterHandlerImpl,
                                >,
                            )
                        };
                        callback
                    },
                    clientd: handler
                        .map(|m| m.as_raw())
                        .unwrap_or_else(|| std::ptr::null_mut()),
                };
                let inner_ptr: *mut aeron_on_unavailable_counter_pair_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_on_unavailable_counter_pair_t)
                );
                let inst: aeron_on_unavailable_counter_pair_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_on_unavailable_counter_pair_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_on_unavailable_counter_pair_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn handler(&self) -> aeron_on_unavailable_counter_t {
        self.handler.into()
    }
    #[inline]
    pub fn clientd(&self) -> *mut ::std::os::raw::c_void {
        self.clientd.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_on_unavailable_counter_pair_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_on_unavailable_counter_pair_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_on_unavailable_counter_pair_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUnavailableCounterPair {
    type Target = aeron_on_unavailable_counter_pair_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_on_unavailable_counter_pair_t> for AeronUnavailableCounterPair {
    #[inline]
    fn from(value: *mut aeron_on_unavailable_counter_pair_t) -> Self {
        AeronUnavailableCounterPair {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUnavailableCounterPair> for *mut aeron_on_unavailable_counter_pair_t {
    #[inline]
    fn from(value: AeronUnavailableCounterPair) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUnavailableCounterPair> for *mut aeron_on_unavailable_counter_pair_t {
    #[inline]
    fn from(value: &AeronUnavailableCounterPair) -> Self {
        value.get_inner()
    }
}
impl From<AeronUnavailableCounterPair> for aeron_on_unavailable_counter_pair_t {
    #[inline]
    fn from(value: AeronUnavailableCounterPair) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_on_unavailable_counter_pair_t> for AeronUnavailableCounterPair {
    #[inline]
    fn from(value: *const aeron_on_unavailable_counter_pair_t) -> Self {
        AeronUnavailableCounterPair {
            inner: CResource::Borrowed(value as *mut aeron_on_unavailable_counter_pair_t),
        }
    }
}
impl From<aeron_on_unavailable_counter_pair_t> for AeronUnavailableCounterPair {
    #[inline]
    fn from(value: aeron_on_unavailable_counter_pair_t) -> Self {
        AeronUnavailableCounterPair {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronUnavailableCounterPair {
    fn default() -> Self {
        AeronUnavailableCounterPair::new_zeroed_on_heap()
    }
}
impl AeronUnavailableCounterPair {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronOptionHeader {
    inner: CResource<aeron_option_header_t>,
}
impl core::fmt::Debug for AeronOptionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronOptionHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronOptionHeader))
                .field("inner", &self.inner)
                .field(stringify!(option_length), &self.option_length())
                .field(stringify!(type_), &self.type_())
                .finish()
        }
    }
}
impl AeronOptionHeader {
    #[inline]
    pub fn new(option_length: u16, type_: u16) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_option_header_t {
                    option_length: option_length.into(),
                    type_: type_.into(),
                };
                let inner_ptr: *mut aeron_option_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_option_header_t)
                );
                let inst: aeron_option_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_option_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_option_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn option_length(&self) -> u16 {
        self.option_length.into()
    }
    #[inline]
    pub fn type_(&self) -> u16 {
        self.type_.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_option_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_option_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_option_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronOptionHeader {
    type Target = aeron_option_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_option_header_t> for AeronOptionHeader {
    #[inline]
    fn from(value: *mut aeron_option_header_t) -> Self {
        AeronOptionHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronOptionHeader> for *mut aeron_option_header_t {
    #[inline]
    fn from(value: AeronOptionHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronOptionHeader> for *mut aeron_option_header_t {
    #[inline]
    fn from(value: &AeronOptionHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronOptionHeader> for aeron_option_header_t {
    #[inline]
    fn from(value: AeronOptionHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_option_header_t> for AeronOptionHeader {
    #[inline]
    fn from(value: *const aeron_option_header_t) -> Self {
        AeronOptionHeader {
            inner: CResource::Borrowed(value as *mut aeron_option_header_t),
        }
    }
}
impl From<aeron_option_header_t> for AeronOptionHeader {
    #[inline]
    fn from(value: aeron_option_header_t) -> Self {
        AeronOptionHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronOptionHeader {
    fn default() -> Self {
        AeronOptionHeader::new_zeroed_on_heap()
    }
}
impl AeronOptionHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronPerThreadError {
    inner: CResource<aeron_per_thread_error_t>,
}
impl core::fmt::Debug for AeronPerThreadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronPerThreadError))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronPerThreadError))
                .field("inner", &self.inner)
                .field(stringify!(offset), &self.offset())
                .finish()
        }
    }
}
impl AeronPerThreadError {
    #[inline]
    pub fn new(
        errcode: ::std::os::raw::c_int,
        offset: usize,
        errmsg: [::std::os::raw::c_char; 8192usize],
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_per_thread_error_t {
                    errcode: errcode.into(),
                    offset: offset.into(),
                    errmsg: errmsg.into(),
                };
                let inner_ptr: *mut aeron_per_thread_error_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_per_thread_error_t)
                );
                let inst: aeron_per_thread_error_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_per_thread_error_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_per_thread_error_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn errcode(&self) -> ::std::os::raw::c_int {
        self.errcode.into()
    }
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset.into()
    }
    #[inline]
    pub fn errmsg(&self) -> [::std::os::raw::c_char; 8192usize] {
        self.errmsg.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_per_thread_error_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_per_thread_error_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_per_thread_error_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronPerThreadError {
    type Target = aeron_per_thread_error_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_per_thread_error_t> for AeronPerThreadError {
    #[inline]
    fn from(value: *mut aeron_per_thread_error_t) -> Self {
        AeronPerThreadError {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronPerThreadError> for *mut aeron_per_thread_error_t {
    #[inline]
    fn from(value: AeronPerThreadError) -> Self {
        value.get_inner()
    }
}
impl From<&AeronPerThreadError> for *mut aeron_per_thread_error_t {
    #[inline]
    fn from(value: &AeronPerThreadError) -> Self {
        value.get_inner()
    }
}
impl From<AeronPerThreadError> for aeron_per_thread_error_t {
    #[inline]
    fn from(value: AeronPerThreadError) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_per_thread_error_t> for AeronPerThreadError {
    #[inline]
    fn from(value: *const aeron_per_thread_error_t) -> Self {
        AeronPerThreadError {
            inner: CResource::Borrowed(value as *mut aeron_per_thread_error_t),
        }
    }
}
impl From<aeron_per_thread_error_t> for AeronPerThreadError {
    #[inline]
    fn from(value: aeron_per_thread_error_t) -> Self {
        AeronPerThreadError {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronPerThreadError {
    fn default() -> Self {
        AeronPerThreadError::new_zeroed_on_heap()
    }
}
impl AeronPerThreadError {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[doc = "Configuration for a publication that does not change during it's lifetime."]
#[derive(Clone)]
pub struct AeronPublicationConstants {
    inner: CResource<aeron_publication_constants_t>,
}
impl core::fmt::Debug for AeronPublicationConstants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronPublicationConstants))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronPublicationConstants))
                .field("inner", &self.inner)
                .field(
                    stringify!(original_registration_id),
                    &self.original_registration_id(),
                )
                .field(stringify!(registration_id), &self.registration_id())
                .field(
                    stringify!(max_possible_position),
                    &self.max_possible_position(),
                )
                .field(
                    stringify!(position_bits_to_shift),
                    &self.position_bits_to_shift(),
                )
                .field(stringify!(term_buffer_length), &self.term_buffer_length())
                .field(stringify!(max_message_length), &self.max_message_length())
                .field(stringify!(max_payload_length), &self.max_payload_length())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(
                    stringify!(publication_limit_counter_id),
                    &self.publication_limit_counter_id(),
                )
                .field(
                    stringify!(channel_status_indicator_id),
                    &self.channel_status_indicator_id(),
                )
                .finish()
        }
    }
}
impl AeronPublicationConstants {
    #[inline]
    pub fn new(
        channel: &std::ffi::CStr,
        original_registration_id: i64,
        registration_id: i64,
        max_possible_position: i64,
        position_bits_to_shift: usize,
        term_buffer_length: usize,
        max_message_length: usize,
        max_payload_length: usize,
        stream_id: i32,
        session_id: i32,
        initial_term_id: i32,
        publication_limit_counter_id: i32,
        channel_status_indicator_id: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_publication_constants_t {
                    channel: channel.as_ptr(),
                    original_registration_id: original_registration_id.into(),
                    registration_id: registration_id.into(),
                    max_possible_position: max_possible_position.into(),
                    position_bits_to_shift: position_bits_to_shift.into(),
                    term_buffer_length: term_buffer_length.into(),
                    max_message_length: max_message_length.into(),
                    max_payload_length: max_payload_length.into(),
                    stream_id: stream_id.into(),
                    session_id: session_id.into(),
                    initial_term_id: initial_term_id.into(),
                    publication_limit_counter_id: publication_limit_counter_id.into(),
                    channel_status_indicator_id: channel_status_indicator_id.into(),
                };
                let inner_ptr: *mut aeron_publication_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_publication_constants_t)
                );
                let inst: aeron_publication_constants_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_publication_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_publication_constants_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn channel(&self) -> &str {
        if self.channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.channel).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn original_registration_id(&self) -> i64 {
        self.original_registration_id.into()
    }
    #[inline]
    pub fn registration_id(&self) -> i64 {
        self.registration_id.into()
    }
    #[inline]
    pub fn max_possible_position(&self) -> i64 {
        self.max_possible_position.into()
    }
    #[inline]
    pub fn position_bits_to_shift(&self) -> usize {
        self.position_bits_to_shift.into()
    }
    #[inline]
    pub fn term_buffer_length(&self) -> usize {
        self.term_buffer_length.into()
    }
    #[inline]
    pub fn max_message_length(&self) -> usize {
        self.max_message_length.into()
    }
    #[inline]
    pub fn max_payload_length(&self) -> usize {
        self.max_payload_length.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn publication_limit_counter_id(&self) -> i32 {
        self.publication_limit_counter_id.into()
    }
    #[inline]
    pub fn channel_status_indicator_id(&self) -> i32 {
        self.channel_status_indicator_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_publication_constants_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_publication_constants_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_publication_constants_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronPublicationConstants {
    type Target = aeron_publication_constants_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_publication_constants_t> for AeronPublicationConstants {
    #[inline]
    fn from(value: *mut aeron_publication_constants_t) -> Self {
        AeronPublicationConstants {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronPublicationConstants> for *mut aeron_publication_constants_t {
    #[inline]
    fn from(value: AeronPublicationConstants) -> Self {
        value.get_inner()
    }
}
impl From<&AeronPublicationConstants> for *mut aeron_publication_constants_t {
    #[inline]
    fn from(value: &AeronPublicationConstants) -> Self {
        value.get_inner()
    }
}
impl From<AeronPublicationConstants> for aeron_publication_constants_t {
    #[inline]
    fn from(value: AeronPublicationConstants) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_publication_constants_t> for AeronPublicationConstants {
    #[inline]
    fn from(value: *const aeron_publication_constants_t) -> Self {
        AeronPublicationConstants {
            inner: CResource::Borrowed(value as *mut aeron_publication_constants_t),
        }
    }
}
impl From<aeron_publication_constants_t> for AeronPublicationConstants {
    #[inline]
    fn from(value: aeron_publication_constants_t) -> Self {
        AeronPublicationConstants {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronPublicationConstants {
    fn default() -> Self {
        AeronPublicationConstants::new_zeroed_on_heap()
    }
}
impl AeronPublicationConstants {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronPublicationErrorValues {
    inner: CResource<aeron_publication_error_values_t>,
}
impl core::fmt::Debug for AeronPublicationErrorValues {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronPublicationErrorValues))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronPublicationErrorValues))
                .field("inner", &self.inner)
                .field(stringify!(registration_id), &self.registration_id())
                .field(
                    stringify!(destination_registration_id),
                    &self.destination_registration_id(),
                )
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(receiver_id), &self.receiver_id())
                .field(stringify!(group_tag), &self.group_tag())
                .field(stringify!(address_type), &self.address_type())
                .field(stringify!(source_port), &self.source_port())
                .field(stringify!(error_code), &self.error_code())
                .field(
                    stringify!(error_message_length),
                    &self.error_message_length(),
                )
                .finish()
        }
    }
}
impl AeronPublicationErrorValues {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_publication_error_values_t)
                );
                let inst: aeron_publication_error_values_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_publication_error_values_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_publication_error_values_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn registration_id(&self) -> i64 {
        self.registration_id.into()
    }
    #[inline]
    pub fn destination_registration_id(&self) -> i64 {
        self.destination_registration_id.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn receiver_id(&self) -> i64 {
        self.receiver_id.into()
    }
    #[inline]
    pub fn group_tag(&self) -> i64 {
        self.group_tag.into()
    }
    #[inline]
    pub fn address_type(&self) -> i16 {
        self.address_type.into()
    }
    #[inline]
    pub fn source_port(&self) -> u16 {
        self.source_port.into()
    }
    #[inline]
    pub fn source_address(&self) -> [u8; 16usize] {
        self.source_address.into()
    }
    #[inline]
    pub fn error_code(&self) -> i32 {
        self.error_code.into()
    }
    #[inline]
    pub fn error_message_length(&self) -> i32 {
        self.error_message_length.into()
    }
    #[inline]
    pub fn error_message(&self) -> [u8; 1usize] {
        self.error_message.into()
    }
    #[inline]
    #[doc = "Delete a instance of `AeronPublicationErrorValues` that was created when making a copy"]
    #[doc = " (aeron_publication_error_values_copy). This should not be use on the pointer received via the aeron_frame_handler_t."]
    pub fn delete(&self) -> () {
        unsafe {
            let result = aeron_publication_error_values_delete(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_publication_error_values_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_publication_error_values_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_publication_error_values_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronPublicationErrorValues {
    type Target = aeron_publication_error_values_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_publication_error_values_t> for AeronPublicationErrorValues {
    #[inline]
    fn from(value: *mut aeron_publication_error_values_t) -> Self {
        AeronPublicationErrorValues {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronPublicationErrorValues> for *mut aeron_publication_error_values_t {
    #[inline]
    fn from(value: AeronPublicationErrorValues) -> Self {
        value.get_inner()
    }
}
impl From<&AeronPublicationErrorValues> for *mut aeron_publication_error_values_t {
    #[inline]
    fn from(value: &AeronPublicationErrorValues) -> Self {
        value.get_inner()
    }
}
impl From<AeronPublicationErrorValues> for aeron_publication_error_values_t {
    #[inline]
    fn from(value: AeronPublicationErrorValues) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_publication_error_values_t> for AeronPublicationErrorValues {
    #[inline]
    fn from(value: *const aeron_publication_error_values_t) -> Self {
        AeronPublicationErrorValues {
            inner: CResource::Borrowed(value as *mut aeron_publication_error_values_t),
        }
    }
}
impl From<aeron_publication_error_values_t> for AeronPublicationErrorValues {
    #[inline]
    fn from(value: aeron_publication_error_values_t) -> Self {
        AeronPublicationErrorValues {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronPublication {
    inner: CResource<aeron_publication_t>,
}
impl core::fmt::Debug for AeronPublication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronPublication))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronPublication))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronPublication {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_publication_t)
                );
                let inst: aeron_publication_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_publication_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            Some(|c| unsafe { aeron_publication_is_closed(c) }),
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_publication_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Non-blocking publish of a buffer containing a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` to publish."]
    #[doc = " \n - `length` of the buffer."]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn offer<AeronReservedValueSupplierHandlerImpl: AeronReservedValueSupplierCallback>(
        &self,
        buffer: &[u8],
        reserved_value_supplier: Option<&Handler<AeronReservedValueSupplierHandlerImpl>>,
    ) -> i64 {
        unsafe {
            let result = aeron_publication_offer(
                self.get_inner(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                {
                    let callback: aeron_reserved_value_supplier_t =
                        if reserved_value_supplier.is_none() {
                            None
                        } else {
                            Some(
                                aeron_reserved_value_supplier_t_callback::<
                                    AeronReservedValueSupplierHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                reserved_value_supplier
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish of a buffer containing a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` to publish."]
    #[doc = " \n - `length` of the buffer."]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn offer_once<AeronReservedValueSupplierHandlerImpl: FnMut(*mut u8, usize) -> i64>(
        &self,
        buffer: &[u8],
        mut reserved_value_supplier: AeronReservedValueSupplierHandlerImpl,
    ) -> i64 {
        unsafe {
            let result = aeron_publication_offer(
                self.get_inner(),
                buffer.as_ptr() as *mut _,
                buffer.len(),
                Some(
                    aeron_reserved_value_supplier_t_callback_for_once_closure::<
                        AeronReservedValueSupplierHandlerImpl,
                    >,
                ),
                &mut reserved_value_supplier as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish by gathering buffer vectors into a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `iov` array for the vectors"]
    #[doc = " \n - `iovcnt` of the number of vectors"]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn offerv<AeronReservedValueSupplierHandlerImpl: AeronReservedValueSupplierCallback>(
        &self,
        iov: &AeronIovec,
        iovcnt: usize,
        reserved_value_supplier: Option<&Handler<AeronReservedValueSupplierHandlerImpl>>,
    ) -> i64 {
        unsafe {
            let result = aeron_publication_offerv(
                self.get_inner(),
                iov.get_inner(),
                iovcnt.into(),
                {
                    let callback: aeron_reserved_value_supplier_t =
                        if reserved_value_supplier.is_none() {
                            None
                        } else {
                            Some(
                                aeron_reserved_value_supplier_t_callback::<
                                    AeronReservedValueSupplierHandlerImpl,
                                >,
                            )
                        };
                    callback
                },
                reserved_value_supplier
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Non-blocking publish by gathering buffer vectors into a message."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `iov` array for the vectors"]
    #[doc = " \n - `iovcnt` of the number of vectors"]
    #[doc = " \n - `reserved_value_supplier` to use for setting the reserved value field or NULL."]
    #[doc = " \n - `clientd` to pass to the reserved_value_supplier."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn offerv_once<AeronReservedValueSupplierHandlerImpl: FnMut(*mut u8, usize) -> i64>(
        &self,
        iov: &AeronIovec,
        iovcnt: usize,
        mut reserved_value_supplier: AeronReservedValueSupplierHandlerImpl,
    ) -> i64 {
        unsafe {
            let result = aeron_publication_offerv(
                self.get_inner(),
                iov.get_inner(),
                iovcnt.into(),
                Some(
                    aeron_reserved_value_supplier_t_callback_for_once_closure::<
                        AeronReservedValueSupplierHandlerImpl,
                    >,
                ),
                &mut reserved_value_supplier as *mut _ as *mut std::os::raw::c_void,
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Try to claim a range in the publication log into which a message can be written with zero copy semantics."]
    #[doc = " Once the message has been written then aeron_buffer_claim_commit should be called thus making it available."]
    #[doc = " A claim length cannot be greater than max payload length."]
    #[doc = " \n"]
    #[doc = " <b>Note:</b> This method can only be used for message lengths less than MTU length minus header."]
    #[doc = " If the claim is held for more than the aeron.publication.unblock.timeout system property then the driver will"]
    #[doc = " assume the publication thread is dead and will unblock the claim thus allowing other threads to make progress"]
    #[doc = " and other claims to be sent to reach end-of-stream (EOS)."]
    #[doc = ""]
    #[doc = " @code"]
    #[doc = " `AeronBufferClaim` buffer_claim;"]
    #[doc = ""]
    #[doc = " if (`AeronPublication`ry_claim(publication, length, &buffer_claim) > 0L)"]
    #[doc = " {"]
    #[doc = "     // work with buffer_claim->data directly."]
    #[doc = "     aeron_buffer_claim_commit(&buffer_claim);"]
    #[doc = " }"]
    #[doc = " @endcode"]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `length` of the message."]
    #[doc = " \n - `buffer_claim` to be populated if the claim succeeds."]
    #[doc = " \n# Return\n the new stream position otherwise a negative error value."]
    pub fn try_claim(&self, length: usize, buffer_claim: &AeronBufferClaim) -> i64 {
        unsafe {
            let result = aeron_publication_try_claim(
                self.get_inner(),
                length.into(),
                buffer_claim.get_inner(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the status of the media channel for this publication."]
    #[doc = " \n"]
    #[doc = " The status will be ERRORED (-1) if a socket exception occurs on setup and ACTIVE (1) if all is well."]
    #[doc = ""]
    #[doc = " \n# Return\n 1 for ACTIVE, -1 for ERRORED"]
    pub fn channel_status(&self) -> i64 {
        unsafe {
            let result = aeron_publication_channel_status(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Has the publication closed?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if this publication is closed."]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_publication_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Has the publication seen an active Subscriber recently?"]
    #[doc = ""]
    #[doc = " \n# Return\n true if this publication has recently seen an active subscriber otherwise false."]
    pub fn is_connected(&self) -> bool {
        unsafe {
            let result = aeron_publication_is_connected(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a publication."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `publication` to get the constants for."]
    #[doc = " \n - `constants` structure to fill in with the constants"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn constants(&self, constants: &AeronPublicationConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_publication_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a publication."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronPublicationConstants, AeronCError> {
        let result = AeronPublicationConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Get the current position to which the publication has advanced for this stream."]
    #[doc = ""]
    #[doc = " \n# Return\n the current position to which the publication has advanced for this stream or a negative error value."]
    pub fn position(&self) -> i64 {
        unsafe {
            let result = aeron_publication_position(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the position limit beyond which this publication will be back pressured."]
    #[doc = ""]
    #[doc = " This should only be used as a guide to determine when back pressure is likely to be applied."]
    #[doc = ""]
    #[doc = " \n# Return\n the position limit beyond which this publication will be back pressured or a negative error value."]
    pub fn position_limit(&self) -> i64 {
        unsafe {
            let result = aeron_publication_position_limit(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Asynchronously close the publication. Will callback on the on_complete notification when the publication is closed."]
    #[doc = " The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the publication has been closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn close<AeronNotificationHandlerImpl: AeronNotificationCallback>(
        &self,
        on_close_complete: Option<&Handler<AeronNotificationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_publication_close(
                self.get_inner(),
                {
                    let callback: aeron_notification_t = if on_close_complete.is_none() {
                        None
                    } else {
                        Some(aeron_notification_t_callback::<AeronNotificationHandlerImpl>)
                    };
                    callback
                },
                on_close_complete
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Asynchronously close the publication. Will callback on the on_complete notification when the publication is closed."]
    #[doc = " The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the publication has been closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn close_once<AeronNotificationHandlerImpl: FnMut() -> ()>(
        &self,
        mut on_close_complete: AeronNotificationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_publication_close(
                self.get_inner(),
                Some(
                    aeron_notification_t_callback_for_once_closure::<AeronNotificationHandlerImpl>,
                ),
                &mut on_close_complete as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Get the publication's channel"]
    #[doc = ""]
    #[doc = " \n# Return\n channel uri string"]
    pub fn channel(&self) -> &str {
        unsafe {
            let result = aeron_publication_channel(self.get_inner());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Get the publication's stream id"]
    #[doc = ""]
    #[doc = " \n# Return\n stream id"]
    pub fn stream_id(&self) -> i32 {
        unsafe {
            let result = aeron_publication_stream_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the publication's session id"]
    #[doc = " \n# Return\n session id"]
    pub fn session_id(&self) -> i32 {
        unsafe {
            let result = aeron_publication_session_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get all of the local socket addresses for this publication. Typically only one representing the control address."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `address_vec` to hold the received addresses"]
    #[doc = " \n - `address_vec_len` available length of the vector to hold the addresses"]
    #[doc = " \n# Return\n number of addresses found or -1 if there is an error."]
    #[doc = " @see aeron_subscription_local_sockaddrs"]
    pub fn local_sockaddrs(
        &self,
        address_vec: &AeronIovec,
        address_vec_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_publication_local_sockaddrs(
                self.get_inner(),
                address_vec.get_inner(),
                address_vec_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn image_location(
        dst: *mut ::std::os::raw::c_char,
        length: usize,
        aeron_dir: &std::ffi::CStr,
        correlation_id: i64,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_publication_image_location(
                dst.into(),
                length.into(),
                aeron_dir.as_ptr(),
                correlation_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_publication_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_publication_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_publication_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronPublication {
    type Target = aeron_publication_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_publication_t> for AeronPublication {
    #[inline]
    fn from(value: *mut aeron_publication_t) -> Self {
        AeronPublication {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronPublication> for *mut aeron_publication_t {
    #[inline]
    fn from(value: AeronPublication) -> Self {
        value.get_inner()
    }
}
impl From<&AeronPublication> for *mut aeron_publication_t {
    #[inline]
    fn from(value: &AeronPublication) -> Self {
        value.get_inner()
    }
}
impl From<AeronPublication> for aeron_publication_t {
    #[inline]
    fn from(value: AeronPublication) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_publication_t> for AeronPublication {
    #[inline]
    fn from(value: *const aeron_publication_t) -> Self {
        AeronPublication {
            inner: CResource::Borrowed(value as *mut aeron_publication_t),
        }
    }
}
impl From<aeron_publication_t> for AeronPublication {
    #[inline]
    fn from(value: aeron_publication_t) -> Self {
        AeronPublication {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronPublication {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronPublication));
                    let result = self.close_with_no_args();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronPublication));
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct AeronResolutionHeaderIpv4 {
    inner: CResource<aeron_resolution_header_ipv4_t>,
}
impl core::fmt::Debug for AeronResolutionHeaderIpv4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronResolutionHeaderIpv4))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronResolutionHeaderIpv4))
                .field("inner", &self.inner)
                .field(stringify!(resolution_header), &self.resolution_header())
                .field(stringify!(name_length), &self.name_length())
                .finish()
        }
    }
}
impl AeronResolutionHeaderIpv4 {
    #[inline]
    pub fn new(
        resolution_header: AeronResolutionHeader,
        addr: [u8; 4usize],
        name_length: i16,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_resolution_header_ipv4_t {
                    resolution_header: resolution_header.into(),
                    addr: addr.into(),
                    name_length: name_length.into(),
                };
                let inner_ptr: *mut aeron_resolution_header_ipv4_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_resolution_header_ipv4_t)
                );
                let inst: aeron_resolution_header_ipv4_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_resolution_header_ipv4_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_resolution_header_ipv4_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn resolution_header(&self) -> AeronResolutionHeader {
        self.resolution_header.into()
    }
    #[inline]
    pub fn addr(&self) -> [u8; 4usize] {
        self.addr.into()
    }
    #[inline]
    pub fn name_length(&self) -> i16 {
        self.name_length.into()
    }
    #[inline]
    pub fn aeron_res_header_entry_length_ipv4(&self) -> usize {
        unsafe {
            let result = aeron_res_header_entry_length_ipv4(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_resolution_header_ipv4_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_resolution_header_ipv4_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_resolution_header_ipv4_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronResolutionHeaderIpv4 {
    type Target = aeron_resolution_header_ipv4_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_resolution_header_ipv4_t> for AeronResolutionHeaderIpv4 {
    #[inline]
    fn from(value: *mut aeron_resolution_header_ipv4_t) -> Self {
        AeronResolutionHeaderIpv4 {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronResolutionHeaderIpv4> for *mut aeron_resolution_header_ipv4_t {
    #[inline]
    fn from(value: AeronResolutionHeaderIpv4) -> Self {
        value.get_inner()
    }
}
impl From<&AeronResolutionHeaderIpv4> for *mut aeron_resolution_header_ipv4_t {
    #[inline]
    fn from(value: &AeronResolutionHeaderIpv4) -> Self {
        value.get_inner()
    }
}
impl From<AeronResolutionHeaderIpv4> for aeron_resolution_header_ipv4_t {
    #[inline]
    fn from(value: AeronResolutionHeaderIpv4) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_resolution_header_ipv4_t> for AeronResolutionHeaderIpv4 {
    #[inline]
    fn from(value: *const aeron_resolution_header_ipv4_t) -> Self {
        AeronResolutionHeaderIpv4 {
            inner: CResource::Borrowed(value as *mut aeron_resolution_header_ipv4_t),
        }
    }
}
impl From<aeron_resolution_header_ipv4_t> for AeronResolutionHeaderIpv4 {
    #[inline]
    fn from(value: aeron_resolution_header_ipv4_t) -> Self {
        AeronResolutionHeaderIpv4 {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronResolutionHeaderIpv4 {
    fn default() -> Self {
        AeronResolutionHeaderIpv4::new_zeroed_on_heap()
    }
}
impl AeronResolutionHeaderIpv4 {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronResolutionHeaderIpv6 {
    inner: CResource<aeron_resolution_header_ipv6_t>,
}
impl core::fmt::Debug for AeronResolutionHeaderIpv6 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronResolutionHeaderIpv6))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronResolutionHeaderIpv6))
                .field("inner", &self.inner)
                .field(stringify!(resolution_header), &self.resolution_header())
                .field(stringify!(name_length), &self.name_length())
                .finish()
        }
    }
}
impl AeronResolutionHeaderIpv6 {
    #[inline]
    pub fn new(
        resolution_header: AeronResolutionHeader,
        addr: [u8; 16usize],
        name_length: i16,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_resolution_header_ipv6_t {
                    resolution_header: resolution_header.into(),
                    addr: addr.into(),
                    name_length: name_length.into(),
                };
                let inner_ptr: *mut aeron_resolution_header_ipv6_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_resolution_header_ipv6_t)
                );
                let inst: aeron_resolution_header_ipv6_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_resolution_header_ipv6_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_resolution_header_ipv6_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn resolution_header(&self) -> AeronResolutionHeader {
        self.resolution_header.into()
    }
    #[inline]
    pub fn addr(&self) -> [u8; 16usize] {
        self.addr.into()
    }
    #[inline]
    pub fn name_length(&self) -> i16 {
        self.name_length.into()
    }
    #[inline]
    pub fn aeron_res_header_entry_length_ipv6(&self) -> usize {
        unsafe {
            let result = aeron_res_header_entry_length_ipv6(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_resolution_header_ipv6_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_resolution_header_ipv6_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_resolution_header_ipv6_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronResolutionHeaderIpv6 {
    type Target = aeron_resolution_header_ipv6_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_resolution_header_ipv6_t> for AeronResolutionHeaderIpv6 {
    #[inline]
    fn from(value: *mut aeron_resolution_header_ipv6_t) -> Self {
        AeronResolutionHeaderIpv6 {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronResolutionHeaderIpv6> for *mut aeron_resolution_header_ipv6_t {
    #[inline]
    fn from(value: AeronResolutionHeaderIpv6) -> Self {
        value.get_inner()
    }
}
impl From<&AeronResolutionHeaderIpv6> for *mut aeron_resolution_header_ipv6_t {
    #[inline]
    fn from(value: &AeronResolutionHeaderIpv6) -> Self {
        value.get_inner()
    }
}
impl From<AeronResolutionHeaderIpv6> for aeron_resolution_header_ipv6_t {
    #[inline]
    fn from(value: AeronResolutionHeaderIpv6) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_resolution_header_ipv6_t> for AeronResolutionHeaderIpv6 {
    #[inline]
    fn from(value: *const aeron_resolution_header_ipv6_t) -> Self {
        AeronResolutionHeaderIpv6 {
            inner: CResource::Borrowed(value as *mut aeron_resolution_header_ipv6_t),
        }
    }
}
impl From<aeron_resolution_header_ipv6_t> for AeronResolutionHeaderIpv6 {
    #[inline]
    fn from(value: aeron_resolution_header_ipv6_t) -> Self {
        AeronResolutionHeaderIpv6 {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronResolutionHeaderIpv6 {
    fn default() -> Self {
        AeronResolutionHeaderIpv6::new_zeroed_on_heap()
    }
}
impl AeronResolutionHeaderIpv6 {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronResolutionHeader {
    inner: CResource<aeron_resolution_header_t>,
}
impl core::fmt::Debug for AeronResolutionHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronResolutionHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronResolutionHeader))
                .field("inner", &self.inner)
                .field(stringify!(udp_port), &self.udp_port())
                .field(stringify!(age_in_ms), &self.age_in_ms())
                .finish()
        }
    }
}
impl AeronResolutionHeader {
    #[inline]
    pub fn new(
        res_type: i8,
        res_flags: u8,
        udp_port: u16,
        age_in_ms: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_resolution_header_t {
                    res_type: res_type.into(),
                    res_flags: res_flags.into(),
                    udp_port: udp_port.into(),
                    age_in_ms: age_in_ms.into(),
                };
                let inner_ptr: *mut aeron_resolution_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_resolution_header_t)
                );
                let inst: aeron_resolution_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_resolution_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_resolution_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn res_type(&self) -> i8 {
        self.res_type.into()
    }
    #[inline]
    pub fn res_flags(&self) -> u8 {
        self.res_flags.into()
    }
    #[inline]
    pub fn udp_port(&self) -> u16 {
        self.udp_port.into()
    }
    #[inline]
    pub fn age_in_ms(&self) -> i32 {
        self.age_in_ms.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_resolution_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_resolution_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_resolution_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronResolutionHeader {
    type Target = aeron_resolution_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_resolution_header_t> for AeronResolutionHeader {
    #[inline]
    fn from(value: *mut aeron_resolution_header_t) -> Self {
        AeronResolutionHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronResolutionHeader> for *mut aeron_resolution_header_t {
    #[inline]
    fn from(value: AeronResolutionHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronResolutionHeader> for *mut aeron_resolution_header_t {
    #[inline]
    fn from(value: &AeronResolutionHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronResolutionHeader> for aeron_resolution_header_t {
    #[inline]
    fn from(value: AeronResolutionHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_resolution_header_t> for AeronResolutionHeader {
    #[inline]
    fn from(value: *const aeron_resolution_header_t) -> Self {
        AeronResolutionHeader {
            inner: CResource::Borrowed(value as *mut aeron_resolution_header_t),
        }
    }
}
impl From<aeron_resolution_header_t> for AeronResolutionHeader {
    #[inline]
    fn from(value: aeron_resolution_header_t) -> Self {
        AeronResolutionHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronResolutionHeader {
    fn default() -> Self {
        AeronResolutionHeader::new_zeroed_on_heap()
    }
}
impl AeronResolutionHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronResponseSetupHeader {
    inner: CResource<aeron_response_setup_header_t>,
}
impl core::fmt::Debug for AeronResponseSetupHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronResponseSetupHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronResponseSetupHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(response_session_id), &self.response_session_id())
                .finish()
        }
    }
}
impl AeronResponseSetupHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        session_id: i32,
        stream_id: i32,
        response_session_id: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_response_setup_header_t {
                    frame_header: frame_header.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    response_session_id: response_session_id.into(),
                };
                let inner_ptr: *mut aeron_response_setup_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_response_setup_header_t)
                );
                let inst: aeron_response_setup_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_response_setup_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_response_setup_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn response_session_id(&self) -> i32 {
        self.response_session_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_response_setup_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_response_setup_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_response_setup_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronResponseSetupHeader {
    type Target = aeron_response_setup_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_response_setup_header_t> for AeronResponseSetupHeader {
    #[inline]
    fn from(value: *mut aeron_response_setup_header_t) -> Self {
        AeronResponseSetupHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronResponseSetupHeader> for *mut aeron_response_setup_header_t {
    #[inline]
    fn from(value: AeronResponseSetupHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronResponseSetupHeader> for *mut aeron_response_setup_header_t {
    #[inline]
    fn from(value: &AeronResponseSetupHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronResponseSetupHeader> for aeron_response_setup_header_t {
    #[inline]
    fn from(value: AeronResponseSetupHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_response_setup_header_t> for AeronResponseSetupHeader {
    #[inline]
    fn from(value: *const aeron_response_setup_header_t) -> Self {
        AeronResponseSetupHeader {
            inner: CResource::Borrowed(value as *mut aeron_response_setup_header_t),
        }
    }
}
impl From<aeron_response_setup_header_t> for AeronResponseSetupHeader {
    #[inline]
    fn from(value: aeron_response_setup_header_t) -> Self {
        AeronResponseSetupHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronResponseSetupHeader {
    fn default() -> Self {
        AeronResponseSetupHeader::new_zeroed_on_heap()
    }
}
impl AeronResponseSetupHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronRttmHeader {
    inner: CResource<aeron_rttm_header_t>,
}
impl core::fmt::Debug for AeronRttmHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronRttmHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronRttmHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(echo_timestamp), &self.echo_timestamp())
                .field(stringify!(reception_delta), &self.reception_delta())
                .field(stringify!(receiver_id), &self.receiver_id())
                .finish()
        }
    }
}
impl AeronRttmHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        session_id: i32,
        stream_id: i32,
        echo_timestamp: i64,
        reception_delta: i64,
        receiver_id: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_rttm_header_t {
                    frame_header: frame_header.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    echo_timestamp: echo_timestamp.into(),
                    reception_delta: reception_delta.into(),
                    receiver_id: receiver_id.into(),
                };
                let inner_ptr: *mut aeron_rttm_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_rttm_header_t)
                );
                let inst: aeron_rttm_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_rttm_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_rttm_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn echo_timestamp(&self) -> i64 {
        self.echo_timestamp.into()
    }
    #[inline]
    pub fn reception_delta(&self) -> i64 {
        self.reception_delta.into()
    }
    #[inline]
    pub fn receiver_id(&self) -> i64 {
        self.receiver_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_rttm_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_rttm_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_rttm_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronRttmHeader {
    type Target = aeron_rttm_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_rttm_header_t> for AeronRttmHeader {
    #[inline]
    fn from(value: *mut aeron_rttm_header_t) -> Self {
        AeronRttmHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronRttmHeader> for *mut aeron_rttm_header_t {
    #[inline]
    fn from(value: AeronRttmHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronRttmHeader> for *mut aeron_rttm_header_t {
    #[inline]
    fn from(value: &AeronRttmHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronRttmHeader> for aeron_rttm_header_t {
    #[inline]
    fn from(value: AeronRttmHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_rttm_header_t> for AeronRttmHeader {
    #[inline]
    fn from(value: *const aeron_rttm_header_t) -> Self {
        AeronRttmHeader {
            inner: CResource::Borrowed(value as *mut aeron_rttm_header_t),
        }
    }
}
impl From<aeron_rttm_header_t> for AeronRttmHeader {
    #[inline]
    fn from(value: aeron_rttm_header_t) -> Self {
        AeronRttmHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronRttmHeader {
    fn default() -> Self {
        AeronRttmHeader::new_zeroed_on_heap()
    }
}
impl AeronRttmHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronSetupHeader {
    inner: CResource<aeron_setup_header_t>,
}
impl core::fmt::Debug for AeronSetupHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronSetupHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronSetupHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(term_offset), &self.term_offset())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(initial_term_id), &self.initial_term_id())
                .field(stringify!(active_term_id), &self.active_term_id())
                .field(stringify!(term_length), &self.term_length())
                .field(stringify!(mtu), &self.mtu())
                .field(stringify!(ttl), &self.ttl())
                .finish()
        }
    }
}
impl AeronSetupHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        term_offset: i32,
        session_id: i32,
        stream_id: i32,
        initial_term_id: i32,
        active_term_id: i32,
        term_length: i32,
        mtu: i32,
        ttl: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_setup_header_t {
                    frame_header: frame_header.into(),
                    term_offset: term_offset.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    initial_term_id: initial_term_id.into(),
                    active_term_id: active_term_id.into(),
                    term_length: term_length.into(),
                    mtu: mtu.into(),
                    ttl: ttl.into(),
                };
                let inner_ptr: *mut aeron_setup_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_setup_header_t)
                );
                let inst: aeron_setup_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_setup_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_setup_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn term_offset(&self) -> i32 {
        self.term_offset.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn initial_term_id(&self) -> i32 {
        self.initial_term_id.into()
    }
    #[inline]
    pub fn active_term_id(&self) -> i32 {
        self.active_term_id.into()
    }
    #[inline]
    pub fn term_length(&self) -> i32 {
        self.term_length.into()
    }
    #[inline]
    pub fn mtu(&self) -> i32 {
        self.mtu.into()
    }
    #[inline]
    pub fn ttl(&self) -> i32 {
        self.ttl.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_setup_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_setup_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_setup_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronSetupHeader {
    type Target = aeron_setup_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_setup_header_t> for AeronSetupHeader {
    #[inline]
    fn from(value: *mut aeron_setup_header_t) -> Self {
        AeronSetupHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronSetupHeader> for *mut aeron_setup_header_t {
    #[inline]
    fn from(value: AeronSetupHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronSetupHeader> for *mut aeron_setup_header_t {
    #[inline]
    fn from(value: &AeronSetupHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronSetupHeader> for aeron_setup_header_t {
    #[inline]
    fn from(value: AeronSetupHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_setup_header_t> for AeronSetupHeader {
    #[inline]
    fn from(value: *const aeron_setup_header_t) -> Self {
        AeronSetupHeader {
            inner: CResource::Borrowed(value as *mut aeron_setup_header_t),
        }
    }
}
impl From<aeron_setup_header_t> for AeronSetupHeader {
    #[inline]
    fn from(value: aeron_setup_header_t) -> Self {
        AeronSetupHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronSetupHeader {
    fn default() -> Self {
        AeronSetupHeader::new_zeroed_on_heap()
    }
}
impl AeronSetupHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronStatusMessageHeader {
    inner: CResource<aeron_status_message_header_t>,
}
impl core::fmt::Debug for AeronStatusMessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronStatusMessageHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronStatusMessageHeader))
                .field("inner", &self.inner)
                .field(stringify!(frame_header), &self.frame_header())
                .field(stringify!(session_id), &self.session_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(stringify!(consumption_term_id), &self.consumption_term_id())
                .field(
                    stringify!(consumption_term_offset),
                    &self.consumption_term_offset(),
                )
                .field(stringify!(receiver_window), &self.receiver_window())
                .field(stringify!(receiver_id), &self.receiver_id())
                .finish()
        }
    }
}
impl AeronStatusMessageHeader {
    #[inline]
    pub fn new(
        frame_header: AeronFrameHeader,
        session_id: i32,
        stream_id: i32,
        consumption_term_id: i32,
        consumption_term_offset: i32,
        receiver_window: i32,
        receiver_id: i64,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_status_message_header_t {
                    frame_header: frame_header.into(),
                    session_id: session_id.into(),
                    stream_id: stream_id.into(),
                    consumption_term_id: consumption_term_id.into(),
                    consumption_term_offset: consumption_term_offset.into(),
                    receiver_window: receiver_window.into(),
                    receiver_id: receiver_id.into(),
                };
                let inner_ptr: *mut aeron_status_message_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_status_message_header_t)
                );
                let inst: aeron_status_message_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_status_message_header_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_status_message_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn frame_header(&self) -> AeronFrameHeader {
        self.frame_header.into()
    }
    #[inline]
    pub fn session_id(&self) -> i32 {
        self.session_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn consumption_term_id(&self) -> i32 {
        self.consumption_term_id.into()
    }
    #[inline]
    pub fn consumption_term_offset(&self) -> i32 {
        self.consumption_term_offset.into()
    }
    #[inline]
    pub fn receiver_window(&self) -> i32 {
        self.receiver_window.into()
    }
    #[inline]
    pub fn receiver_id(&self) -> i64 {
        self.receiver_id.into()
    }
    #[inline]
    pub fn aeron_udp_protocol_group_tag(&self) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_udp_protocol_group_tag(self.get_inner(), &mut mut_result);
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_status_message_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_status_message_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_status_message_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronStatusMessageHeader {
    type Target = aeron_status_message_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_status_message_header_t> for AeronStatusMessageHeader {
    #[inline]
    fn from(value: *mut aeron_status_message_header_t) -> Self {
        AeronStatusMessageHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronStatusMessageHeader> for *mut aeron_status_message_header_t {
    #[inline]
    fn from(value: AeronStatusMessageHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronStatusMessageHeader> for *mut aeron_status_message_header_t {
    #[inline]
    fn from(value: &AeronStatusMessageHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronStatusMessageHeader> for aeron_status_message_header_t {
    #[inline]
    fn from(value: AeronStatusMessageHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_status_message_header_t> for AeronStatusMessageHeader {
    #[inline]
    fn from(value: *const aeron_status_message_header_t) -> Self {
        AeronStatusMessageHeader {
            inner: CResource::Borrowed(value as *mut aeron_status_message_header_t),
        }
    }
}
impl From<aeron_status_message_header_t> for AeronStatusMessageHeader {
    #[inline]
    fn from(value: aeron_status_message_header_t) -> Self {
        AeronStatusMessageHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronStatusMessageHeader {
    fn default() -> Self {
        AeronStatusMessageHeader::new_zeroed_on_heap()
    }
}
impl AeronStatusMessageHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronStatusMessageOptionalHeader {
    inner: CResource<aeron_status_message_optional_header_t>,
}
impl core::fmt::Debug for AeronStatusMessageOptionalHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronStatusMessageOptionalHeader))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronStatusMessageOptionalHeader))
                .field("inner", &self.inner)
                .field(stringify!(group_tag), &self.group_tag())
                .finish()
        }
    }
}
impl AeronStatusMessageOptionalHeader {
    #[inline]
    pub fn new(group_tag: i64) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_status_message_optional_header_t {
                    group_tag: group_tag.into(),
                };
                let inner_ptr: *mut aeron_status_message_optional_header_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_status_message_optional_header_t)
                );
                let inst: aeron_status_message_optional_header_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_status_message_optional_header_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_status_message_optional_header_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn group_tag(&self) -> i64 {
        self.group_tag.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_status_message_optional_header_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_status_message_optional_header_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_status_message_optional_header_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronStatusMessageOptionalHeader {
    type Target = aeron_status_message_optional_header_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_status_message_optional_header_t> for AeronStatusMessageOptionalHeader {
    #[inline]
    fn from(value: *mut aeron_status_message_optional_header_t) -> Self {
        AeronStatusMessageOptionalHeader {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronStatusMessageOptionalHeader> for *mut aeron_status_message_optional_header_t {
    #[inline]
    fn from(value: AeronStatusMessageOptionalHeader) -> Self {
        value.get_inner()
    }
}
impl From<&AeronStatusMessageOptionalHeader> for *mut aeron_status_message_optional_header_t {
    #[inline]
    fn from(value: &AeronStatusMessageOptionalHeader) -> Self {
        value.get_inner()
    }
}
impl From<AeronStatusMessageOptionalHeader> for aeron_status_message_optional_header_t {
    #[inline]
    fn from(value: AeronStatusMessageOptionalHeader) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_status_message_optional_header_t> for AeronStatusMessageOptionalHeader {
    #[inline]
    fn from(value: *const aeron_status_message_optional_header_t) -> Self {
        AeronStatusMessageOptionalHeader {
            inner: CResource::Borrowed(value as *mut aeron_status_message_optional_header_t),
        }
    }
}
impl From<aeron_status_message_optional_header_t> for AeronStatusMessageOptionalHeader {
    #[inline]
    fn from(value: aeron_status_message_optional_header_t) -> Self {
        AeronStatusMessageOptionalHeader {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronStatusMessageOptionalHeader {
    fn default() -> Self {
        AeronStatusMessageOptionalHeader::new_zeroed_on_heap()
    }
}
impl AeronStatusMessageOptionalHeader {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronStrToPtrHashMapKey {
    inner: CResource<aeron_str_to_ptr_hash_map_key_t>,
}
impl core::fmt::Debug for AeronStrToPtrHashMapKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronStrToPtrHashMapKey))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronStrToPtrHashMapKey))
                .field("inner", &self.inner)
                .field(stringify!(hash_code), &self.hash_code())
                .field(stringify!(str_length), &self.str_length())
                .finish()
        }
    }
}
impl AeronStrToPtrHashMapKey {
    #[inline]
    pub fn new(
        str_: &std::ffi::CStr,
        hash_code: u64,
        str_length: usize,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_str_to_ptr_hash_map_key_t {
                    str_: str_.as_ptr(),
                    hash_code: hash_code.into(),
                    str_length: str_length.into(),
                };
                let inner_ptr: *mut aeron_str_to_ptr_hash_map_key_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_str_to_ptr_hash_map_key_t)
                );
                let inst: aeron_str_to_ptr_hash_map_key_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_str_to_ptr_hash_map_key_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_str_to_ptr_hash_map_key_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn str_(&self) -> &str {
        if self.str_.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.str_).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn hash_code(&self) -> u64 {
        self.hash_code.into()
    }
    #[inline]
    pub fn str_length(&self) -> usize {
        self.str_length.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_str_to_ptr_hash_map_key_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_str_to_ptr_hash_map_key_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_str_to_ptr_hash_map_key_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronStrToPtrHashMapKey {
    type Target = aeron_str_to_ptr_hash_map_key_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_str_to_ptr_hash_map_key_t> for AeronStrToPtrHashMapKey {
    #[inline]
    fn from(value: *mut aeron_str_to_ptr_hash_map_key_t) -> Self {
        AeronStrToPtrHashMapKey {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronStrToPtrHashMapKey> for *mut aeron_str_to_ptr_hash_map_key_t {
    #[inline]
    fn from(value: AeronStrToPtrHashMapKey) -> Self {
        value.get_inner()
    }
}
impl From<&AeronStrToPtrHashMapKey> for *mut aeron_str_to_ptr_hash_map_key_t {
    #[inline]
    fn from(value: &AeronStrToPtrHashMapKey) -> Self {
        value.get_inner()
    }
}
impl From<AeronStrToPtrHashMapKey> for aeron_str_to_ptr_hash_map_key_t {
    #[inline]
    fn from(value: AeronStrToPtrHashMapKey) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_str_to_ptr_hash_map_key_t> for AeronStrToPtrHashMapKey {
    #[inline]
    fn from(value: *const aeron_str_to_ptr_hash_map_key_t) -> Self {
        AeronStrToPtrHashMapKey {
            inner: CResource::Borrowed(value as *mut aeron_str_to_ptr_hash_map_key_t),
        }
    }
}
impl From<aeron_str_to_ptr_hash_map_key_t> for AeronStrToPtrHashMapKey {
    #[inline]
    fn from(value: aeron_str_to_ptr_hash_map_key_t) -> Self {
        AeronStrToPtrHashMapKey {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronStrToPtrHashMapKey {
    fn default() -> Self {
        AeronStrToPtrHashMapKey::new_zeroed_on_heap()
    }
}
impl AeronStrToPtrHashMapKey {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronStrToPtrHashMap {
    inner: CResource<aeron_str_to_ptr_hash_map_t>,
}
impl core::fmt::Debug for AeronStrToPtrHashMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronStrToPtrHashMap))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronStrToPtrHashMap))
                .field("inner", &self.inner)
                .field(stringify!(load_factor), &self.load_factor())
                .field(stringify!(capacity), &self.capacity())
                .field(stringify!(size), &self.size())
                .field(stringify!(resize_threshold), &self.resize_threshold())
                .finish()
        }
    }
}
impl AeronStrToPtrHashMap {
    #[inline]
    pub fn new(
        keys: &AeronStrToPtrHashMapKey,
        values: *mut *mut ::std::os::raw::c_void,
        load_factor: f32,
        capacity: usize,
        size: usize,
        resize_threshold: usize,
    ) -> Result<Self, AeronCError> {
        let keys_copy = keys.clone();
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_str_to_ptr_hash_map_t {
                    keys: keys.into(),
                    values: values.into(),
                    load_factor: load_factor.into(),
                    capacity: capacity.into(),
                    size: size.into(),
                    resize_threshold: resize_threshold.into(),
                };
                let inner_ptr: *mut aeron_str_to_ptr_hash_map_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_str_to_ptr_hash_map_t)
                );
                let inst: aeron_str_to_ptr_hash_map_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_str_to_ptr_hash_map_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_str_to_ptr_hash_map_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn keys(&self) -> AeronStrToPtrHashMapKey {
        self.keys.into()
    }
    #[inline]
    pub fn values(&self) -> *mut *mut ::std::os::raw::c_void {
        self.values.into()
    }
    #[inline]
    pub fn load_factor(&self) -> f32 {
        self.load_factor.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn size(&self) -> usize {
        self.size.into()
    }
    #[inline]
    pub fn resize_threshold(&self) -> usize {
        self.resize_threshold.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_str_to_ptr_hash_map_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_str_to_ptr_hash_map_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_str_to_ptr_hash_map_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronStrToPtrHashMap {
    type Target = aeron_str_to_ptr_hash_map_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_str_to_ptr_hash_map_t> for AeronStrToPtrHashMap {
    #[inline]
    fn from(value: *mut aeron_str_to_ptr_hash_map_t) -> Self {
        AeronStrToPtrHashMap {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronStrToPtrHashMap> for *mut aeron_str_to_ptr_hash_map_t {
    #[inline]
    fn from(value: AeronStrToPtrHashMap) -> Self {
        value.get_inner()
    }
}
impl From<&AeronStrToPtrHashMap> for *mut aeron_str_to_ptr_hash_map_t {
    #[inline]
    fn from(value: &AeronStrToPtrHashMap) -> Self {
        value.get_inner()
    }
}
impl From<AeronStrToPtrHashMap> for aeron_str_to_ptr_hash_map_t {
    #[inline]
    fn from(value: AeronStrToPtrHashMap) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_str_to_ptr_hash_map_t> for AeronStrToPtrHashMap {
    #[inline]
    fn from(value: *const aeron_str_to_ptr_hash_map_t) -> Self {
        AeronStrToPtrHashMap {
            inner: CResource::Borrowed(value as *mut aeron_str_to_ptr_hash_map_t),
        }
    }
}
impl From<aeron_str_to_ptr_hash_map_t> for AeronStrToPtrHashMap {
    #[inline]
    fn from(value: aeron_str_to_ptr_hash_map_t) -> Self {
        AeronStrToPtrHashMap {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronStrToPtrHashMap {
    fn default() -> Self {
        AeronStrToPtrHashMap::new_zeroed_on_heap()
    }
}
impl AeronStrToPtrHashMap {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronSubscriptionConstants {
    inner: CResource<aeron_subscription_constants_t>,
}
impl core::fmt::Debug for AeronSubscriptionConstants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronSubscriptionConstants))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronSubscriptionConstants))
                .field("inner", &self.inner)
                .field(stringify!(registration_id), &self.registration_id())
                .field(stringify!(stream_id), &self.stream_id())
                .field(
                    stringify!(channel_status_indicator_id),
                    &self.channel_status_indicator_id(),
                )
                .finish()
        }
    }
}
impl AeronSubscriptionConstants {
    #[inline]
    pub fn new(
        channel: &std::ffi::CStr,
        on_available_image: aeron_on_available_image_t,
        on_unavailable_image: aeron_on_unavailable_image_t,
        registration_id: i64,
        stream_id: i32,
        channel_status_indicator_id: i32,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_subscription_constants_t {
                    channel: channel.as_ptr(),
                    on_available_image: on_available_image.into(),
                    on_unavailable_image: on_unavailable_image.into(),
                    registration_id: registration_id.into(),
                    stream_id: stream_id.into(),
                    channel_status_indicator_id: channel_status_indicator_id.into(),
                };
                let inner_ptr: *mut aeron_subscription_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_subscription_constants_t)
                );
                let inst: aeron_subscription_constants_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_subscription_constants_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_subscription_constants_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn channel(&self) -> &str {
        if self.channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.channel).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn on_available_image(&self) -> aeron_on_available_image_t {
        self.on_available_image.into()
    }
    #[inline]
    pub fn on_unavailable_image(&self) -> aeron_on_unavailable_image_t {
        self.on_unavailable_image.into()
    }
    #[inline]
    pub fn registration_id(&self) -> i64 {
        self.registration_id.into()
    }
    #[inline]
    pub fn stream_id(&self) -> i32 {
        self.stream_id.into()
    }
    #[inline]
    pub fn channel_status_indicator_id(&self) -> i32 {
        self.channel_status_indicator_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_subscription_constants_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_subscription_constants_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_subscription_constants_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronSubscriptionConstants {
    type Target = aeron_subscription_constants_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_subscription_constants_t> for AeronSubscriptionConstants {
    #[inline]
    fn from(value: *mut aeron_subscription_constants_t) -> Self {
        AeronSubscriptionConstants {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronSubscriptionConstants> for *mut aeron_subscription_constants_t {
    #[inline]
    fn from(value: AeronSubscriptionConstants) -> Self {
        value.get_inner()
    }
}
impl From<&AeronSubscriptionConstants> for *mut aeron_subscription_constants_t {
    #[inline]
    fn from(value: &AeronSubscriptionConstants) -> Self {
        value.get_inner()
    }
}
impl From<AeronSubscriptionConstants> for aeron_subscription_constants_t {
    #[inline]
    fn from(value: AeronSubscriptionConstants) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_subscription_constants_t> for AeronSubscriptionConstants {
    #[inline]
    fn from(value: *const aeron_subscription_constants_t) -> Self {
        AeronSubscriptionConstants {
            inner: CResource::Borrowed(value as *mut aeron_subscription_constants_t),
        }
    }
}
impl From<aeron_subscription_constants_t> for AeronSubscriptionConstants {
    #[inline]
    fn from(value: aeron_subscription_constants_t) -> Self {
        AeronSubscriptionConstants {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronSubscriptionConstants {
    fn default() -> Self {
        AeronSubscriptionConstants::new_zeroed_on_heap()
    }
}
impl AeronSubscriptionConstants {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronSubscription {
    inner: CResource<aeron_subscription_t>,
}
impl core::fmt::Debug for AeronSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronSubscription))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronSubscription))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronSubscription {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_subscription_t)
                );
                let inst: aeron_subscription_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_subscription_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            Some(|c| unsafe { aeron_subscription_is_closed(c) }),
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_subscription_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    #[doc = "Poll the images under the subscription for available message fragments."]
    #[doc = " \n"]
    #[doc = " Each fragment read will be a whole message if it is under MTU length. If larger than MTU then it will come"]
    #[doc = " as a series of fragments ordered within a session."]
    #[doc = " \n"]
    #[doc = " To assemble messages that span multiple fragments then use `AeronFragmentAssembler`."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` for handling each message fragment as it is read."]
    #[doc = " \n - `fragment_limit` number of message fragments to limit when polling across multiple images."]
    #[doc = " \n# Return\n the number of fragments received or -1 for error."]
    pub fn poll<AeronFragmentHandlerHandlerImpl: AeronFragmentHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronFragmentHandlerHandlerImpl>>,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_poll(
                self.get_inner(),
                {
                    let callback: aeron_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_fragment_handler_t_callback::<AeronFragmentHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the images under the subscription for available message fragments."]
    #[doc = " \n"]
    #[doc = " Each fragment read will be a whole message if it is under MTU length. If larger than MTU then it will come"]
    #[doc = " as a series of fragments ordered within a session."]
    #[doc = " \n"]
    #[doc = " To assemble messages that span multiple fragments then use `AeronFragmentAssembler`."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` for handling each message fragment as it is read."]
    #[doc = " \n - `fragment_limit` number of message fragments to limit when polling across multiple images."]
    #[doc = " \n# Return\n the number of fragments received or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn poll_once<AeronFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> ()>(
        &self,
        mut handler: AeronFragmentHandlerHandlerImpl,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_poll(
                self.get_inner(),
                Some(
                    aeron_fragment_handler_t_callback_for_once_closure::<
                        AeronFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll in a controlled manner the images under the subscription for available message fragments."]
    #[doc = " Control is applied to fragments in the stream. If more fragments can be read on another stream"]
    #[doc = " they will even if BREAK or ABORT is returned from the fragment handler."]
    #[doc = " \n"]
    #[doc = " Each fragment read will be a whole message if it is under MTU length. If larger than MTU then it will come"]
    #[doc = " as a series of fragments ordered within a session."]
    #[doc = " \n"]
    #[doc = " To assemble messages that span multiple fragments then use `AeronControlledFragmentAssembler`."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` for handling each message fragment as it is read."]
    #[doc = " \n - `fragment_limit` number of message fragments to limit when polling across multiple images."]
    #[doc = " \n# Return\n the number of fragments received or -1 for error."]
    pub fn controlled_poll<
        AeronControlledFragmentHandlerHandlerImpl: AeronControlledFragmentHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronControlledFragmentHandlerHandlerImpl>>,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_controlled_poll(
                self.get_inner(),
                {
                    let callback: aeron_controlled_fragment_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_controlled_fragment_handler_t_callback::<
                                AeronControlledFragmentHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll in a controlled manner the images under the subscription for available message fragments."]
    #[doc = " Control is applied to fragments in the stream. If more fragments can be read on another stream"]
    #[doc = " they will even if BREAK or ABORT is returned from the fragment handler."]
    #[doc = " \n"]
    #[doc = " Each fragment read will be a whole message if it is under MTU length. If larger than MTU then it will come"]
    #[doc = " as a series of fragments ordered within a session."]
    #[doc = " \n"]
    #[doc = " To assemble messages that span multiple fragments then use `AeronControlledFragmentAssembler`."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` for handling each message fragment as it is read."]
    #[doc = " \n - `fragment_limit` number of message fragments to limit when polling across multiple images."]
    #[doc = " \n# Return\n the number of fragments received or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn controlled_poll_once<
        AeronControlledFragmentHandlerHandlerImpl: FnMut(&[u8], AeronHeader) -> aeron_controlled_fragment_handler_action_t,
    >(
        &self,
        mut handler: AeronControlledFragmentHandlerHandlerImpl,
        fragment_limit: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_controlled_poll(
                self.get_inner(),
                Some(
                    aeron_controlled_fragment_handler_t_callback_for_once_closure::<
                        AeronControlledFragmentHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                fragment_limit.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Poll the images under the subscription for available message fragments in blocks."]
    #[doc = " \n"]
    #[doc = " This method is useful for operations like bulk archiving and messaging indexing."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to receive a block of fragments from each image."]
    #[doc = " \n - `block_length_limit` for each image polled."]
    #[doc = " \n# Return\n the number of bytes consumed or -1 for error."]
    pub fn block_poll<AeronBlockHandlerHandlerImpl: AeronBlockHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronBlockHandlerHandlerImpl>>,
        block_length_limit: usize,
    ) -> ::std::os::raw::c_long {
        unsafe {
            let result = aeron_subscription_block_poll(
                self.get_inner(),
                {
                    let callback: aeron_block_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_block_handler_t_callback::<AeronBlockHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                block_length_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Poll the images under the subscription for available message fragments in blocks."]
    #[doc = " \n"]
    #[doc = " This method is useful for operations like bulk archiving and messaging indexing."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to receive a block of fragments from each image."]
    #[doc = " \n - `block_length_limit` for each image polled."]
    #[doc = " \n# Return\n the number of bytes consumed or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn block_poll_once<AeronBlockHandlerHandlerImpl: FnMut(&[u8], i32, i32) -> ()>(
        &self,
        mut handler: AeronBlockHandlerHandlerImpl,
        block_length_limit: usize,
    ) -> ::std::os::raw::c_long {
        unsafe {
            let result = aeron_subscription_block_poll(
                self.get_inner(),
                Some(
                    aeron_block_handler_t_callback_for_once_closure::<AeronBlockHandlerHandlerImpl>,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                block_length_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = "Is this subscription connected by having at least one open publication image."]
    #[doc = ""]
    #[doc = " \n# Return\n true if this subscription connected by having at least one open publication image."]
    pub fn is_connected(&self) -> bool {
        unsafe {
            let result = aeron_subscription_is_connected(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a subscription."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `subscription` to get the constants for."]
    #[doc = " \n - `constants` structure to fill in with the constants"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn constants(&self, constants: &AeronSubscriptionConstants) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_constants(self.get_inner(), constants.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Fill in a structure with the constants in use by a subscription."]
    #[doc = ""]
    pub fn get_constants(&self) -> Result<AeronSubscriptionConstants, AeronCError> {
        let result = AeronSubscriptionConstants::new_zeroed_on_stack();
        self.constants(&result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Count of images associated to this subscription."]
    #[doc = ""]
    #[doc = " \n# Return\n count of count associated to this subscription or -1 for error."]
    pub fn image_count(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_image_count(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return the image associated with the given session_id under the given subscription."]
    #[doc = ""]
    #[doc = " Note: the returned image is considered retained by the application and thus must be released via"]
    #[doc = " aeron_image_release when finished or if the image becomes unavailable."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `session_id` associated with the image."]
    #[doc = " \n# Return\n image associated with the given session_id or NULL if no image exists."]
    pub fn image_by_session_id(&self, session_id: i32) -> AeronImage {
        unsafe {
            let result =
                aeron_subscription_image_by_session_id(self.get_inner(), session_id.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Return the image at the given index."]
    #[doc = ""]
    #[doc = " Note: the returned image is considered retained by the application and thus must be released via"]
    #[doc = " aeron_image_release when finished or if the image becomes unavailable."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `index` for the image."]
    #[doc = " \n# Return\n image at the given index or NULL if no image exists."]
    pub fn image_at_index(&self, index: usize) -> AeronImage {
        unsafe {
            let result = aeron_subscription_image_at_index(self.get_inner(), index.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Iterate over the images for this subscription calling the given function."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `handler` to be called for each image."]
    #[doc = " \n - `clientd` to be passed to the handler."]
    pub fn for_each_image(
        &self,
        handler: ::std::option::Option<
            unsafe extern "C" fn(image: *mut aeron_image_t, clientd: *mut ::std::os::raw::c_void),
        >,
        clientd: *mut ::std::os::raw::c_void,
    ) -> () {
        unsafe {
            let result =
                aeron_subscription_for_each_image(self.get_inner(), handler.into(), clientd.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Retain the given image for access in the application."]
    #[doc = ""]
    #[doc = " Note: A retain call must have a corresponding release call."]
    #[doc = " Note: Subscriptions are not threadsafe and should not be shared between subscribers."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `subscription` that image is part of."]
    #[doc = " \n - `image` to retain"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn image_retain(&self, image: &AeronImage) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_image_retain(self.get_inner(), image.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Release the given image and relinquish desire to use the image directly."]
    #[doc = ""]
    #[doc = " Note: Subscriptions are not threadsafe and should not be shared between subscribers."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `subscription` that image is part of."]
    #[doc = " \n - `image` to release"]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn image_release(&self, image: &AeronImage) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_image_release(self.get_inner(), image.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Is the subscription closed."]
    #[doc = ""]
    #[doc = " \n# Return\n true if it has been closed otherwise false."]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_subscription_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Get the status of the media channel for this subscription."]
    #[doc = " \n"]
    #[doc = " The status will be ERRORED (-1) if a socket exception occurs on setup and ACTIVE (1) if all is well."]
    #[doc = ""]
    #[doc = " \n# Return\n 1 for ACTIVE, -1 for ERRORED"]
    pub fn channel_status(&self) -> i64 {
        unsafe {
            let result = aeron_subscription_channel_status(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Asynchronously close the subscription. Will callback on the on_complete notification when the subscription is"]
    #[doc = " closed. The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the subscription has been closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    pub fn close<AeronNotificationHandlerImpl: AeronNotificationCallback>(
        &self,
        on_close_complete: Option<&Handler<AeronNotificationHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_subscription_close(
                self.get_inner(),
                {
                    let callback: aeron_notification_t = if on_close_complete.is_none() {
                        None
                    } else {
                        Some(aeron_notification_t_callback::<AeronNotificationHandlerImpl>)
                    };
                    callback
                },
                on_close_complete
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Asynchronously close the subscription. Will callback on the on_complete notification when the subscription is"]
    #[doc = " closed. The callback is optional, use NULL for the on_complete callback if not required."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `on_close_complete` optional callback to execute once the subscription has been closed and freed. This may"]
    #[doc = " happen on a separate thread, so the caller should ensure that clientd has the appropriate lifetime."]
    #[doc = " \n - `on_close_complete_clientd` parameter to pass to the on_complete callback."]
    #[doc = " \n# Return\n 0 for success or -1 for error."]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn close_once<AeronNotificationHandlerImpl: FnMut() -> ()>(
        &self,
        mut on_close_complete: AeronNotificationHandlerImpl,
    ) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_subscription_close(
                self.get_inner(),
                Some(
                    aeron_notification_t_callback_for_once_closure::<AeronNotificationHandlerImpl>,
                ),
                &mut on_close_complete as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Get all of the local socket addresses for this subscription. Multiple addresses can occur if this is a"]
    #[doc = " multi-destination subscription. Addresses will a string representation in numeric form. IPv6 addresses will be"]
    #[doc = " surrounded by '[' and ']' so that the ':' that separate the parts are distinguishable from the port delimiter."]
    #[doc = " E.g. [fe80::7552:c06e:6bf4:4160]:12345. As of writing the maximum length for a formatted address is 54 bytes"]
    #[doc = " including the NULL terminator. AERON_CLIENT_MAX_LOCAL_ADDRESS_STR_LEN is defined to provide enough space to fit the"]
    #[doc = " returned string. Returned strings will be NULL terminated. If the buffer to hold the address can not hold enough"]
    #[doc = " of the message it will be truncated and the last character will be null."]
    #[doc = ""]
    #[doc = " If the address_vec_len is less the total number of addresses available then the first addresses found up to that"]
    #[doc = " length will be placed into the address_vec. However the function will return the total number of addresses available"]
    #[doc = " so if if that is larger than the input array then the client code may wish to re-query with a larger array to get"]
    #[doc = " them all."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `address_vec` to hold the received addresses"]
    #[doc = " \n - `address_vec_len` available length of the vector to hold the addresses"]
    #[doc = " \n# Return\n number of addresses found or -1 if there is an error."]
    pub fn local_sockaddrs(
        &self,
        address_vec: &AeronIovec,
        address_vec_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_local_sockaddrs(
                self.get_inner(),
                address_vec.get_inner(),
                address_vec_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Retrieves the first local socket address for this subscription. If this is not MDS then it will be the one"]
    #[doc = " representing endpoint for this subscription."]
    #[doc = ""]
    #[doc = " @see aeron_subscription_local_sockaddrs"]
    #[doc = "# Parameters\n \n - `address` for the received address"]
    #[doc = " \n - `address_len` available length for the copied address."]
    #[doc = " \n# Return\n -1 on error, 0 if address not found, 1 if address is found."]
    pub fn resolved_endpoint(
        &self,
        address: &std::ffi::CStr,
        address_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_resolved_endpoint(
                self.get_inner(),
                address.as_ptr(),
                address_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Retrieves the channel URI for this subscription with any wildcard ports filled in. If the channel is not UDP or"]
    #[doc = " does not have a wildcard port (<code>0</code>), then it will return the original URI."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `uri` buffer to hold the resolved uri"]
    #[doc = " \n - `uri_len` length of the buffer"]
    #[doc = " \n# Return\n -1 on failure or the number of bytes written to the buffer (excluding the NULL terminator). Writing is done"]
    #[doc = " on a per key basis, so if the buffer was truncated before writing completed, it will only include the byte count up"]
    #[doc = " to the key that overflowed. However, the invariant that if the number returned >= uri_len, then output will have been"]
    #[doc = " truncated."]
    pub fn try_resolve_channel_endpoint_port(
        &self,
        uri: *mut ::std::os::raw::c_char,
        uri_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_subscription_try_resolve_channel_endpoint_port(
                self.get_inner(),
                uri.into(),
                uri_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Retrieves the channel URI for this subscription with any wildcard ports filled in. If the channel is not UDP or"]
    #[doc = " does not have a wildcard port (<code>0</code>), then it will return the original URI."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `uri` buffer to hold the resolved uri"]
    #[doc = " \n - `uri_len` length of the buffer"]
    #[doc = " \n# Return\n -1 on failure or the number of bytes written to the buffer (excluding the NULL terminator). Writing is done"]
    #[doc = " on a per key basis, so if the buffer was truncated before writing completed, it will only include the byte count up"]
    #[doc = " to the key that overflowed. However, the invariant that if the number returned >= uri_len, then output will have been"]
    #[doc = " truncated."]
    pub fn try_resolve_channel_endpoint_port_as_string(
        &self,
        max_length: usize,
    ) -> Result<String, AeronCError> {
        let mut result = String::with_capacity(max_length);
        self.try_resolve_channel_endpoint_port_into(&mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "Retrieves the channel URI for this subscription with any wildcard ports filled in. If the channel is not UDP or"]
    #[doc = " does not have a wildcard port (<code>0</code>), then it will return the original URI."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `uri` buffer to hold the resolved uri"]
    #[doc = " \n - `uri_len` length of the buffer"]
    #[doc = " \n# Return\n -1 on failure or the number of bytes written to the buffer (excluding the NULL terminator). Writing is done"]
    #[doc = " on a per key basis, so if the buffer was truncated before writing completed, it will only include the byte count up"]
    #[doc = " to the key that overflowed. However, the invariant that if the number returned >= uri_len, then output will have been"]
    #[doc = " truncated."]
    #[doc = "NOTE: allocation friendly method, the string capacity must be set as it will truncate string to capacity it will never grow the string. So if you pass String::new() it will write 0 chars"]
    pub fn try_resolve_channel_endpoint_port_into(
        &self,
        dst_truncate_to_capacity: &mut String,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let capacity = dst_truncate_to_capacity.capacity();
            let vec = dst_truncate_to_capacity.as_mut_vec();
            vec.set_len(capacity);
            let result =
                self.try_resolve_channel_endpoint_port(vec.as_mut_ptr() as *mut _, capacity)?;
            let mut len = 0;
            loop {
                if len == capacity {
                    break;
                }
                let val = vec[len];
                if val == 0 {
                    break;
                }
                len += 1;
            }
            vec.set_len(len);
            Ok(result)
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_subscription_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_subscription_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_subscription_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronSubscription {
    type Target = aeron_subscription_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_subscription_t> for AeronSubscription {
    #[inline]
    fn from(value: *mut aeron_subscription_t) -> Self {
        AeronSubscription {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronSubscription> for *mut aeron_subscription_t {
    #[inline]
    fn from(value: AeronSubscription) -> Self {
        value.get_inner()
    }
}
impl From<&AeronSubscription> for *mut aeron_subscription_t {
    #[inline]
    fn from(value: &AeronSubscription) -> Self {
        value.get_inner()
    }
}
impl From<AeronSubscription> for aeron_subscription_t {
    #[inline]
    fn from(value: AeronSubscription) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_subscription_t> for AeronSubscription {
    #[inline]
    fn from(value: *const aeron_subscription_t) -> Self {
        AeronSubscription {
            inner: CResource::Borrowed(value as *mut aeron_subscription_t),
        }
    }
}
impl From<aeron_subscription_t> for AeronSubscription {
    #[inline]
    fn from(value: aeron_subscription_t) -> Self {
        AeronSubscription {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronSubscription {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronSubscription));
                    let result = self.close_with_no_args();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronSubscription));
                }
            }
        }
    }
}
#[derive(Clone)]
pub struct Aeron {
    inner: CResource<aeron_t>,
    _context: Option<AeronContext>,
}
impl core::fmt::Debug for Aeron {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(Aeron))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(Aeron))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl Aeron {
    #[doc = "Create a `Aeron` client struct and initialize from the `AeronContext` struct."]
    #[doc = ""]
    #[doc = " The given `AeronContext` struct will be used exclusively by the client. Do not reuse between clients."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `context` to use for initialization."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn new(context: &AeronContext) -> Result<Self, AeronCError> {
        let context_copy = context.clone();
        let context: *mut aeron_context_t = context.into();
        let resource_constructor = ManagedCResource::new(
            move |ctx_field| unsafe { aeron_init(ctx_field, context) },
            Some(Box::new(move |ctx_field| unsafe {
                aeron_close(*ctx_field)
            })),
            false,
            Some(|c| unsafe { aeron_is_closed(c) }),
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource_constructor)),
            _context: Some(context_copy),
        })
    }
    #[inline]
    #[doc = "Start an `Aeron`. This may spawn a thread for the Client Conductor."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn start(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_start(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Call the Conductor main do_work duty cycle once."]
    #[doc = ""]
    #[doc = " Client must have been created with use conductor invoker set to true."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn main_do_work(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_main_do_work(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Call the Conductor Idle Strategy."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `work_count` to pass to idle strategy."]
    pub fn main_idle_strategy(&self, work_count: ::std::os::raw::c_int) -> () {
        unsafe {
            let result = aeron_main_idle_strategy(self.get_inner(), work_count.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Close and delete `Aeron` struct."]
    #[doc = ""]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Determines if the client has been closed, e.g. via a driver timeout. Don't call this method after calling"]
    #[doc = " aeron_close as that will have already freed the associated memory."]
    #[doc = ""]
    #[doc = " \n# Return\n true if it has been closed, false otherwise."]
    pub fn is_closed(&self) -> bool {
        unsafe {
            let result = aeron_is_closed(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Call stream_out to print the counter labels and values."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `stream_out` to call for each label and value."]
    pub fn print_counters(
        &self,
        stream_out: ::std::option::Option<
            unsafe extern "C" fn(arg1: *const ::std::os::raw::c_char),
        >,
    ) -> () {
        unsafe {
            let result = aeron_print_counters(self.get_inner(), stream_out.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Return the `AeronContext` that is in use by the given client."]
    #[doc = ""]
    #[doc = " \n# Return\n the `AeronContext` for the given client or NULL for an error."]
    pub fn context(&self) -> AeronContext {
        unsafe {
            let result = aeron_context(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Return the client id in use by the client."]
    #[doc = ""]
    #[doc = " \n# Return\n id value or -1 for an error."]
    pub fn client_id(&self) -> i64 {
        unsafe {
            let result = aeron_client_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Return a unique correlation id from the driver."]
    #[doc = ""]
    #[doc = " \n# Return\n unique correlation id or -1 for an error."]
    pub fn next_correlation_id(&self) -> i64 {
        unsafe {
            let result = aeron_next_correlation_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Return a reference to the counters reader of the given client."]
    #[doc = ""]
    #[doc = " The `AeronCountersReader` is maintained by the client. And should not be freed."]
    #[doc = ""]
    #[doc = " \n# Return\n `AeronCountersReader` or NULL for error."]
    pub fn counters_reader(&self) -> AeronCountersReader {
        unsafe {
            let result = aeron_counters_reader(self.get_inner());
            result.into()
        }
    }
    #[inline]
    #[doc = "Add a handler to be called when a new counter becomes available."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is added by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn add_available_counter_handler(
        &self,
        pair: &AeronAvailableCounterPair,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_add_available_counter_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Remove a previously added handler to be called when a new counter becomes available."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is removed by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn remove_available_counter_handler(
        &self,
        pair: &AeronAvailableCounterPair,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_remove_available_counter_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Add a handler to be called when a new counter becomes unavailable or goes away."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is added by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn add_unavailable_counter_handler(
        &self,
        pair: &AeronUnavailableCounterPair,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_add_unavailable_counter_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Remove a previously added handler to be called when a new counter becomes unavailable or goes away."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is removed by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn remove_unavailable_counter_handler(
        &self,
        pair: &AeronUnavailableCounterPair,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_remove_unavailable_counter_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Add a handler to be called when client is closed."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is added by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn add_close_handler(&self, pair: &AeronCloseClientPair) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_add_close_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Remove a previously added handler to be called when client is closed."]
    #[doc = ""]
    #[doc = " NOTE: This function blocks until the handler is removed by the client conductor thread."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `pair` holding the handler to call and a clientd to pass when called."]
    #[doc = " \n# Return\n 0 for success and -1 for error"]
    pub fn remove_close_handler(&self, pair: &AeronCloseClientPair) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_remove_close_handler(self.get_inner(), pair.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return full version and build string."]
    #[doc = ""]
    #[doc = " \n# Return\n full version and build string."]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn version_full() -> &'static str {
        unsafe {
            let result = aeron_version_full();
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Return version text."]
    #[doc = ""]
    #[doc = " \n# Return\n version text."]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn version_text() -> &'static str {
        unsafe {
            let result = aeron_version_text();
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Return major version number."]
    #[doc = ""]
    #[doc = " \n# Return\n major version number."]
    pub fn version_major() -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_version_major();
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return minor version number."]
    #[doc = ""]
    #[doc = " \n# Return\n minor version number."]
    pub fn version_minor() -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_version_minor();
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return patch version number."]
    #[doc = ""]
    #[doc = " \n# Return\n patch version number."]
    pub fn version_patch() -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_version_patch();
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return the git sha for the current build."]
    #[doc = ""]
    #[doc = " \n# Return\n git version"]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn version_gitsha() -> &'static str {
        unsafe {
            let result = aeron_version_gitsha();
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Return time in nanoseconds for machine. Is not wall clock time."]
    #[doc = ""]
    #[doc = " \n# Return\n nanoseconds since epoch for machine."]
    pub fn nano_clock() -> i64 {
        unsafe {
            let result = aeron_nano_clock();
            result.into()
        }
    }
    #[inline]
    #[doc = "Return time in milliseconds since epoch. Is wall clock time."]
    #[doc = ""]
    #[doc = " \n# Return\n milliseconds since epoch."]
    pub fn epoch_clock() -> i64 {
        unsafe {
            let result = aeron_epoch_clock();
            result.into()
        }
    }
    #[inline]
    #[doc = "Determine if an aeron driver is using a given aeron directory."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `dirname`  for aeron directory"]
    #[doc = " \n - `timeout_ms`  to use to determine activity for aeron directory"]
    #[doc = " \n - `log_func` to call during activity check to log diagnostic information."]
    #[doc = " \n# Return\n true for active driver or false for no active driver."]
    pub fn is_driver_active(
        dirname: &std::ffi::CStr,
        timeout_ms: i64,
        log_func: aeron_log_func_t,
    ) -> bool {
        unsafe {
            let result =
                aeron_is_driver_active(dirname.as_ptr(), timeout_ms.into(), log_func.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "Load properties from a string containing name=value pairs and set appropriate environment variables for the"]
    #[doc = " process so that subsequent calls to aeron_driver_context_init will use those values."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `buffer` containing properties and values."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn properties_buffer_load(buffer: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_properties_buffer_load(buffer.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Load properties file and set appropriate environment variables for the process so that subsequent"]
    #[doc = " calls to aeron_driver_context_init will use those values."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `filename` to load."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn properties_file_load(filename: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_properties_file_load(filename.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Load properties from HTTP URL and set environment variables for the process so that subsequent"]
    #[doc = " calls to aeron_driver_context_init will use those values."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `url` to attempt to retrieve and load."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn properties_http_load(url: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_properties_http_load(url.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Load properties based on URL or filename. If string contains file or http URL, it will attempt"]
    #[doc = " to load properties from a file or http as indicated. If not a URL, then it will try to load the string"]
    #[doc = " as a filename."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `url_or_filename` to load properties from."]
    #[doc = " \n# Return\n 0 for success and -1 for error."]
    pub fn properties_load(url_or_filename: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_properties_load(url_or_filename.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return current aeron error code (errno) for calling thread."]
    #[doc = ""]
    #[doc = " \n# Return\n aeron error code for calling thread."]
    pub fn errcode() -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_errcode();
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = "Return the current aeron error message for calling thread."]
    #[doc = ""]
    #[doc = " \n# Return\n aeron error message for calling thread."]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn errmsg() -> &'static str {
        unsafe {
            let result = aeron_errmsg();
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    #[doc = "Get the default path used by the Aeron media driver."]
    #[doc = ""]
    #[doc = "# Parameters\n \n - `path` buffer to store the path."]
    #[doc = " \n - `path_length` space available in the buffer"]
    #[doc = " \n# Return\n -1 if there is an issue or the number of bytes written to path excluding the terminator <code>\\0</code>. If this"]
    #[doc = " is equal to or greater than the path_length then the path has been truncated."]
    pub fn default_path(
        path: *mut ::std::os::raw::c_char,
        path_length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_default_path(path.into(), path_length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn semantic_version_compose(major: u8, minor: u8, patch: u8) -> i32 {
        unsafe {
            let result = aeron_semantic_version_compose(major.into(), minor.into(), patch.into());
            result.into()
        }
    }
    #[inline]
    pub fn semantic_version_major(version: i32) -> u8 {
        unsafe {
            let result = aeron_semantic_version_major(version.into());
            result.into()
        }
    }
    #[inline]
    pub fn semantic_version_minor(version: i32) -> u8 {
        unsafe {
            let result = aeron_semantic_version_minor(version.into());
            result.into()
        }
    }
    #[inline]
    pub fn semantic_version_patch(version: i32) -> u8 {
        unsafe {
            let result = aeron_semantic_version_patch(version.into());
            result.into()
        }
    }
    #[inline]
    pub fn thread_set_name(role_name: &std::ffi::CStr) -> () {
        unsafe {
            let result = aeron_thread_set_name(role_name.as_ptr());
            result.into()
        }
    }
    #[inline]
    pub fn nano_sleep(nanoseconds: u64) -> () {
        unsafe {
            let result = aeron_nano_sleep(nanoseconds.into());
            result.into()
        }
    }
    #[inline]
    pub fn micro_sleep(microseconds: ::std::os::raw::c_uint) -> () {
        unsafe {
            let result = aeron_micro_sleep(microseconds.into());
            result.into()
        }
    }
    #[inline]
    pub fn thread_set_affinity(
        role_name: &std::ffi::CStr,
        cpu_affinity_no: u8,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_thread_set_affinity(role_name.as_ptr(), cpu_affinity_no.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn randomised_int32() -> i32 {
        unsafe {
            let result = aeron_randomised_int32();
            result.into()
        }
    }
    #[inline]
    pub fn format_date(str_: *mut ::std::os::raw::c_char, count: usize, timestamp: i64) -> () {
        unsafe {
            let result = aeron_format_date(str_.into(), count.into(), timestamp.into());
            result.into()
        }
    }
    #[inline]
    pub fn format_number_to_locale(
        value: ::std::os::raw::c_longlong,
        buffer: *mut ::std::os::raw::c_char,
        buffer_len: usize,
    ) -> *mut ::std::os::raw::c_char {
        unsafe {
            let result =
                aeron_format_number_to_locale(value.into(), buffer.into(), buffer_len.into());
            result.into()
        }
    }
    #[inline]
    pub fn format_to_hex(
        str_: *mut ::std::os::raw::c_char,
        str_length: usize,
        data: *const u8,
        data_len: usize,
    ) -> () {
        unsafe {
            let result =
                aeron_format_to_hex(str_.into(), str_length.into(), data.into(), data_len.into());
            result.into()
        }
    }
    #[inline]
    pub fn set_errno(errcode: ::std::os::raw::c_int) -> () {
        unsafe {
            let result = aeron_set_errno(errcode.into());
            result.into()
        }
    }
    #[inline]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn error_code_str(errcode: ::std::os::raw::c_int) -> &'static str {
        unsafe {
            let result = aeron_error_code_str(errcode.into());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    pub fn err_set(
        errcode: ::std::os::raw::c_int,
        function: &std::ffi::CStr,
        filename: &std::ffi::CStr,
        line_number: ::std::os::raw::c_int,
        format: &std::ffi::CStr,
    ) -> () {
        unsafe {
            let result = aeron_err_set(
                errcode.into(),
                function.as_ptr(),
                filename.as_ptr(),
                line_number.into(),
                format.as_ptr(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn err_append(
        function: &std::ffi::CStr,
        filename: &std::ffi::CStr,
        line_number: ::std::os::raw::c_int,
        format: &std::ffi::CStr,
    ) -> () {
        unsafe {
            let result = aeron_err_append(
                function.as_ptr(),
                filename.as_ptr(),
                line_number.into(),
                format.as_ptr(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn err_clear() -> () {
        unsafe {
            let result = aeron_err_clear();
            result.into()
        }
    }
    #[inline]
    pub fn free(ptr: *mut ::std::os::raw::c_void) -> () {
        unsafe {
            let result = aeron_free(ptr.into());
            result.into()
        }
    }
    #[inline]
    pub fn res_header_entry_length(
        res: *mut ::std::os::raw::c_void,
        remaining: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_res_header_entry_length(res.into(), remaining.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn logbuffer_check_term_length(term_length: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_logbuffer_check_term_length(term_length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn logbuffer_check_page_size(page_size: u64) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_logbuffer_check_page_size(page_size.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn is_directory(path: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_is_directory(path.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn delete_directory(directory: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_delete_directory(directory.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn mkdir_recursive(
        pathname: &std::ffi::CStr,
        permission: ::std::os::raw::c_int,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_mkdir_recursive(pathname.as_ptr(), permission.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn msync(addr: *mut ::std::os::raw::c_void, length: usize) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_msync(addr.into(), length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn delete_file(path: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_delete_file(path.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn file_length(path: &std::ffi::CStr) -> i64 {
        unsafe {
            let result = aeron_file_length(path.as_ptr());
            result.into()
        }
    }
    #[inline]
    pub fn usable_fs_space(path: &std::ffi::CStr) -> u64 {
        unsafe {
            let result = aeron_usable_fs_space(path.as_ptr());
            result.into()
        }
    }
    #[inline]
    pub fn usable_fs_space_disabled(path: &std::ffi::CStr) -> u64 {
        unsafe {
            let result = aeron_usable_fs_space_disabled(path.as_ptr());
            result.into()
        }
    }
    #[inline]
    pub fn ipc_publication_location(
        dst: *mut ::std::os::raw::c_char,
        length: usize,
        aeron_dir: &std::ffi::CStr,
        correlation_id: i64,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_ipc_publication_location(
                dst.into(),
                length.into(),
                aeron_dir.as_ptr(),
                correlation_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn network_publication_location(
        dst: *mut ::std::os::raw::c_char,
        length: usize,
        aeron_dir: &std::ffi::CStr,
        correlation_id: i64,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_network_publication_location(
                dst.into(),
                length.into(),
                aeron_dir.as_ptr(),
                correlation_id.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn temp_filename(filename: *mut ::std::os::raw::c_char, length: usize) -> usize {
        unsafe {
            let result = aeron_temp_filename(filename.into(), length.into());
            result.into()
        }
    }
    #[inline]
    pub fn file_resolve(
        parent: &std::ffi::CStr,
        child: &std::ffi::CStr,
        buffer: *mut ::std::os::raw::c_char,
        buffer_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_file_resolve(
                parent.as_ptr(),
                child.as_ptr(),
                buffer.into(),
                buffer_len.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for Aeron {
    type Target = aeron_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_t> for Aeron {
    #[inline]
    fn from(value: *mut aeron_t) -> Self {
        Aeron {
            inner: CResource::Borrowed(value),
            _context: None,
        }
    }
}
impl From<Aeron> for *mut aeron_t {
    #[inline]
    fn from(value: Aeron) -> Self {
        value.get_inner()
    }
}
impl From<&Aeron> for *mut aeron_t {
    #[inline]
    fn from(value: &Aeron) -> Self {
        value.get_inner()
    }
}
impl From<Aeron> for aeron_t {
    #[inline]
    fn from(value: Aeron) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_t> for Aeron {
    #[inline]
    fn from(value: *const aeron_t) -> Self {
        Aeron {
            inner: CResource::Borrowed(value as *mut aeron_t),
            _context: None,
        }
    }
}
impl From<aeron_t> for Aeron {
    #[inline]
    fn from(value: aeron_t) -> Self {
        Aeron {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
            _context: None,
        }
    }
}
#[derive(Clone)]
pub struct AeronUdpChannelParams {
    inner: CResource<aeron_udp_channel_params_t>,
}
impl core::fmt::Debug for AeronUdpChannelParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUdpChannelParams))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUdpChannelParams))
                .field("inner", &self.inner)
                .field(stringify!(additional_params), &self.additional_params())
                .finish()
        }
    }
}
impl AeronUdpChannelParams {
    #[inline]
    pub fn new(
        endpoint: &std::ffi::CStr,
        bind_interface: &std::ffi::CStr,
        control: &std::ffi::CStr,
        control_mode: &std::ffi::CStr,
        channel_tag: &std::ffi::CStr,
        entity_tag: &std::ffi::CStr,
        ttl: &std::ffi::CStr,
        additional_params: AeronUriParams,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_udp_channel_params_t {
                    endpoint: endpoint.as_ptr(),
                    bind_interface: bind_interface.as_ptr(),
                    control: control.as_ptr(),
                    control_mode: control_mode.as_ptr(),
                    channel_tag: channel_tag.as_ptr(),
                    entity_tag: entity_tag.as_ptr(),
                    ttl: ttl.as_ptr(),
                    additional_params: additional_params.into(),
                };
                let inner_ptr: *mut aeron_udp_channel_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_udp_channel_params_t)
                );
                let inst: aeron_udp_channel_params_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_udp_channel_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_udp_channel_params_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn endpoint(&self) -> &str {
        if self.endpoint.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.endpoint).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn bind_interface(&self) -> &str {
        if self.bind_interface.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.bind_interface)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn control(&self) -> &str {
        if self.control.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.control).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn control_mode(&self) -> &str {
        if self.control_mode.is_null() {
            ""
        } else {
            unsafe {
                std::ffi::CStr::from_ptr(self.control_mode)
                    .to_str()
                    .unwrap()
            }
        }
    }
    #[inline]
    pub fn channel_tag(&self) -> &str {
        if self.channel_tag.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.channel_tag).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn entity_tag(&self) -> &str {
        if self.entity_tag.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.entity_tag).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn ttl(&self) -> &str {
        if self.ttl.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.ttl).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn additional_params(&self) -> AeronUriParams {
        self.additional_params.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_udp_channel_params_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_udp_channel_params_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_udp_channel_params_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUdpChannelParams {
    type Target = aeron_udp_channel_params_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_udp_channel_params_t> for AeronUdpChannelParams {
    #[inline]
    fn from(value: *mut aeron_udp_channel_params_t) -> Self {
        AeronUdpChannelParams {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUdpChannelParams> for *mut aeron_udp_channel_params_t {
    #[inline]
    fn from(value: AeronUdpChannelParams) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUdpChannelParams> for *mut aeron_udp_channel_params_t {
    #[inline]
    fn from(value: &AeronUdpChannelParams) -> Self {
        value.get_inner()
    }
}
impl From<AeronUdpChannelParams> for aeron_udp_channel_params_t {
    #[inline]
    fn from(value: AeronUdpChannelParams) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_udp_channel_params_t> for AeronUdpChannelParams {
    #[inline]
    fn from(value: *const aeron_udp_channel_params_t) -> Self {
        AeronUdpChannelParams {
            inner: CResource::Borrowed(value as *mut aeron_udp_channel_params_t),
        }
    }
}
impl From<aeron_udp_channel_params_t> for AeronUdpChannelParams {
    #[inline]
    fn from(value: aeron_udp_channel_params_t) -> Self {
        AeronUdpChannelParams {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronUdpChannelParams {
    fn default() -> Self {
        AeronUdpChannelParams::new_zeroed_on_heap()
    }
}
impl AeronUdpChannelParams {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronUriParam {
    inner: CResource<aeron_uri_param_t>,
}
impl core::fmt::Debug for AeronUriParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUriParam))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUriParam))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronUriParam {
    #[inline]
    pub fn new(key: &std::ffi::CStr, value: &std::ffi::CStr) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_uri_param_t {
                    key: key.as_ptr(),
                    value: value.as_ptr(),
                };
                let inner_ptr: *mut aeron_uri_param_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_uri_param_t)
                );
                let inst: aeron_uri_param_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_uri_param_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_uri_param_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn key(&self) -> &str {
        if self.key.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.key).to_str().unwrap() }
        }
    }
    #[inline]
    pub fn value(&self) -> &str {
        if self.value.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(self.value).to_str().unwrap() }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_uri_param_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_uri_param_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_uri_param_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUriParam {
    type Target = aeron_uri_param_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_uri_param_t> for AeronUriParam {
    #[inline]
    fn from(value: *mut aeron_uri_param_t) -> Self {
        AeronUriParam {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUriParam> for *mut aeron_uri_param_t {
    #[inline]
    fn from(value: AeronUriParam) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUriParam> for *mut aeron_uri_param_t {
    #[inline]
    fn from(value: &AeronUriParam) -> Self {
        value.get_inner()
    }
}
impl From<AeronUriParam> for aeron_uri_param_t {
    #[inline]
    fn from(value: AeronUriParam) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_uri_param_t> for AeronUriParam {
    #[inline]
    fn from(value: *const aeron_uri_param_t) -> Self {
        AeronUriParam {
            inner: CResource::Borrowed(value as *mut aeron_uri_param_t),
        }
    }
}
impl From<aeron_uri_param_t> for AeronUriParam {
    #[inline]
    fn from(value: aeron_uri_param_t) -> Self {
        AeronUriParam {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronUriParam {
    fn default() -> Self {
        AeronUriParam::new_zeroed_on_heap()
    }
}
impl AeronUriParam {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronUriParams {
    inner: CResource<aeron_uri_params_t>,
}
impl core::fmt::Debug for AeronUriParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUriParams))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUriParams))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .finish()
        }
    }
}
impl AeronUriParams {
    #[inline]
    pub fn new(length: usize, array: &AeronUriParam) -> Result<Self, AeronCError> {
        let array_copy = array.clone();
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_uri_params_t {
                    length: length.into(),
                    array: array.into(),
                };
                let inner_ptr: *mut aeron_uri_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_uri_params_t)
                );
                let inst: aeron_uri_params_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_uri_params_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_uri_params_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn length(&self) -> usize {
        self.length.into()
    }
    #[inline]
    pub fn array(&self) -> AeronUriParam {
        self.array.into()
    }
    #[inline]
    #[doc = "SAFETY: this is static for performance reasons, so you should not store this without copying it!!"]
    pub fn aeron_uri_find_param_value(
        uri_params: *const aeron_uri_params_t,
        key: &std::ffi::CStr,
    ) -> &'static str {
        unsafe {
            let result = aeron_uri_find_param_value(uri_params.into(), key.as_ptr());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_int32(&self, key: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let mut mut_result: i32 = Default::default();
            let err_code = aeron_uri_get_int32(self.get_inner(), key.as_ptr(), &mut mut_result);
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_int64(
        &self,
        key: &std::ffi::CStr,
        default_val: i64,
    ) -> Result<i64, AeronCError> {
        unsafe {
            let mut mut_result: i64 = Default::default();
            let err_code = aeron_uri_get_int64(
                self.get_inner(),
                key.as_ptr(),
                default_val.into(),
                &mut mut_result,
            );
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_bool(&self, key: &std::ffi::CStr) -> Result<bool, AeronCError> {
        unsafe {
            let mut mut_result: bool = Default::default();
            let err_code = aeron_uri_get_bool(self.get_inner(), key.as_ptr(), &mut mut_result);
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_ats(
        &self,
        uri_ats_status: *mut aeron_uri_ats_status_t,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_get_ats(self.get_inner(), uri_ats_status.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_timeout(&self, param_name: &std::ffi::CStr) -> Result<u64, AeronCError> {
        unsafe {
            let mut mut_result: u64 = Default::default();
            let err_code =
                aeron_uri_get_timeout(self.get_inner(), param_name.as_ptr(), &mut mut_result);
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_socket_buf_lengths(
        &self,
        socket_sndbuf_length: &mut usize,
        socket_rcvbuf_length: &mut usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_get_socket_buf_lengths(
                self.get_inner(),
                socket_sndbuf_length as *mut _,
                socket_rcvbuf_length as *mut _,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn aeron_uri_get_receiver_window_length(&self) -> Result<usize, AeronCError> {
        unsafe {
            let mut mut_result: usize = Default::default();
            let err_code = aeron_uri_get_receiver_window_length(self.get_inner(), &mut mut_result);
            if err_code < 0 {
                return Err(AeronCError::from_code(err_code));
            } else {
                return Ok(mut_result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_uri_params_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_uri_params_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_uri_params_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUriParams {
    type Target = aeron_uri_params_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_uri_params_t> for AeronUriParams {
    #[inline]
    fn from(value: *mut aeron_uri_params_t) -> Self {
        AeronUriParams {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUriParams> for *mut aeron_uri_params_t {
    #[inline]
    fn from(value: AeronUriParams) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUriParams> for *mut aeron_uri_params_t {
    #[inline]
    fn from(value: &AeronUriParams) -> Self {
        value.get_inner()
    }
}
impl From<AeronUriParams> for aeron_uri_params_t {
    #[inline]
    fn from(value: AeronUriParams) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_uri_params_t> for AeronUriParams {
    #[inline]
    fn from(value: *const aeron_uri_params_t) -> Self {
        AeronUriParams {
            inner: CResource::Borrowed(value as *mut aeron_uri_params_t),
        }
    }
}
impl From<aeron_uri_params_t> for AeronUriParams {
    #[inline]
    fn from(value: aeron_uri_params_t) -> Self {
        AeronUriParams {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronUriParams {
    fn default() -> Self {
        AeronUriParams::new_zeroed_on_heap()
    }
}
impl AeronUriParams {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronUriStringBuilder {
    inner: CResource<aeron_uri_string_builder_t>,
}
impl core::fmt::Debug for AeronUriStringBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUriStringBuilder))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUriStringBuilder))
                .field("inner", &self.inner)
                .field(stringify!(params), &self.params())
                .field(stringify!(closed), &self.closed())
                .finish()
        }
    }
}
impl AeronUriStringBuilder {
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_uri_string_builder_t)
                );
                let inst: aeron_uri_string_builder_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_uri_string_builder_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_uri_string_builder_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn params(&self) -> AeronStrToPtrHashMap {
        self.params.into()
    }
    #[inline]
    pub fn closed(&self) -> bool {
        self.closed.into()
    }
    #[inline]
    pub fn init_new(&self) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_string_builder_init_new(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn init_on_string(&self, uri: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_string_builder_init_on_string(self.get_inner(), uri.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn close(&self) -> Result<i32, AeronCError> {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_uri_string_builder_close(self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn put(&self, key: &std::ffi::CStr, value: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_uri_string_builder_put(self.get_inner(), key.as_ptr(), value.as_ptr());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn put_int32(&self, key: &std::ffi::CStr, value: i32) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_uri_string_builder_put_int32(self.get_inner(), key.as_ptr(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn put_int64(&self, key: &std::ffi::CStr, value: i64) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_uri_string_builder_put_int64(self.get_inner(), key.as_ptr(), value.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn get(&self, key: &std::ffi::CStr) -> &str {
        unsafe {
            let result = aeron_uri_string_builder_get(self.get_inner(), key.as_ptr());
            if result.is_null() {
                ""
            } else {
                unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() }
            }
        }
    }
    #[inline]
    pub fn sprint(
        &self,
        buffer: *mut ::std::os::raw::c_char,
        buffer_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_uri_string_builder_sprint(self.get_inner(), buffer.into(), buffer_len.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn sprint_as_string(&self, max_length: usize) -> Result<String, AeronCError> {
        let mut result = String::with_capacity(max_length);
        self.sprint_into(&mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "NOTE: allocation friendly method, the string capacity must be set as it will truncate string to capacity it will never grow the string. So if you pass String::new() it will write 0 chars"]
    pub fn sprint_into(&self, dst_truncate_to_capacity: &mut String) -> Result<i32, AeronCError> {
        unsafe {
            let capacity = dst_truncate_to_capacity.capacity();
            let vec = dst_truncate_to_capacity.as_mut_vec();
            vec.set_len(capacity);
            let result = self.sprint(vec.as_mut_ptr() as *mut _, capacity)?;
            let mut len = 0;
            loop {
                if len == capacity {
                    break;
                }
                let val = vec[len];
                if val == 0 {
                    break;
                }
                len += 1;
            }
            vec.set_len(len);
            Ok(result)
        }
    }
    #[inline]
    pub fn set_initial_position(
        &self,
        position: i64,
        initial_term_id: i32,
        term_length: i32,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_string_builder_set_initial_position(
                self.get_inner(),
                position.into(),
                initial_term_id.into(),
                term_length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_uri_string_builder_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_uri_string_builder_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_uri_string_builder_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUriStringBuilder {
    type Target = aeron_uri_string_builder_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_uri_string_builder_t> for AeronUriStringBuilder {
    #[inline]
    fn from(value: *mut aeron_uri_string_builder_t) -> Self {
        AeronUriStringBuilder {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUriStringBuilder> for *mut aeron_uri_string_builder_t {
    #[inline]
    fn from(value: AeronUriStringBuilder) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUriStringBuilder> for *mut aeron_uri_string_builder_t {
    #[inline]
    fn from(value: &AeronUriStringBuilder) -> Self {
        value.get_inner()
    }
}
impl From<AeronUriStringBuilder> for aeron_uri_string_builder_t {
    #[inline]
    fn from(value: AeronUriStringBuilder) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_uri_string_builder_t> for AeronUriStringBuilder {
    #[inline]
    fn from(value: *const aeron_uri_string_builder_t) -> Self {
        AeronUriStringBuilder {
            inner: CResource::Borrowed(value as *mut aeron_uri_string_builder_t),
        }
    }
}
impl From<aeron_uri_string_builder_t> for AeronUriStringBuilder {
    #[inline]
    fn from(value: aeron_uri_string_builder_t) -> Self {
        AeronUriStringBuilder {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
#[derive(Clone)]
pub struct AeronUri {
    inner: CResource<aeron_uri_t>,
}
impl core::fmt::Debug for AeronUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.get().is_null() {
            f.debug_struct(stringify!(AeronUri))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronUri))
                .field("inner", &self.inner)
                .finish()
        }
    }
}
impl AeronUri {
    #[inline]
    pub fn new(
        mutable_uri: [::std::os::raw::c_char; 4096usize],
        type_: aeron_uri_type_t,
        params: aeron_uri_stct__bindgen_ty_1,
    ) -> Result<Self, AeronCError> {
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_uri_t {
                    mutable_uri: mutable_uri.into(),
                    type_: type_.into(),
                    params: params.into(),
                };
                let inner_ptr: *mut aeron_uri_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(r_constructor)),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed_on_heap() -> Self {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(feature = "extra-logging")]
                log::info!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_uri_t)
                );
                let inst: aeron_uri_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_uri_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )
        .unwrap();
        Self {
            inner: CResource::OwnedOnHeap(std::rc::Rc::new(resource)),
        }
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the stack"]
    #[doc = r" _(Use with care)_"]
    pub fn new_zeroed_on_stack() -> Self {
        #[cfg(feature = "extra-logging")]
        log::debug!(
            "creating zeroed empty resource on stack {}",
            stringify!(aeron_uri_t)
        );
        Self {
            inner: CResource::OwnedOnStack(std::mem::MaybeUninit::zeroed()),
        }
    }
    #[inline]
    pub fn mutable_uri(&self) -> [::std::os::raw::c_char; 4096usize] {
        self.mutable_uri.into()
    }
    #[inline]
    pub fn type_(&self) -> aeron_uri_type_t {
        self.type_.into()
    }
    #[inline]
    pub fn params(&self) -> aeron_uri_stct__bindgen_ty_1 {
        self.params.into()
    }
    #[inline]
    pub fn parse_params<AeronUriParseCallbackHandlerImpl: AeronUriParseCallbackCallback>(
        uri: *mut ::std::os::raw::c_char,
        param_func: Option<&Handler<AeronUriParseCallbackHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_parse_params(
                uri.into(),
                {
                    let callback: aeron_uri_parse_callback_t = if param_func.is_none() {
                        None
                    } else {
                        Some(
                            aeron_uri_parse_callback_t_callback::<AeronUriParseCallbackHandlerImpl>,
                        )
                    };
                    callback
                },
                param_func
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn parse_params_once<
        AeronUriParseCallbackHandlerImpl: FnMut(&str, &str) -> ::std::os::raw::c_int,
    >(
        uri: *mut ::std::os::raw::c_char,
        mut param_func: AeronUriParseCallbackHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_parse_params(
                uri.into(),
                Some(
                    aeron_uri_parse_callback_t_callback_for_once_closure::<
                        AeronUriParseCallbackHandlerImpl,
                    >,
                ),
                &mut param_func as *mut _ as *mut std::os::raw::c_void,
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn parse(&self, uri_length: usize, uri: &std::ffi::CStr) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_parse(uri_length.into(), uri.as_ptr(), self.get_inner());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn close(&self) -> () {
        if let Some(inner) = self.inner.as_owned() {
            inner.close_already_called.set(true);
        }
        unsafe {
            let result = aeron_uri_close(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn multicast_ttl(&self) -> u8 {
        unsafe {
            let result = aeron_uri_multicast_ttl(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn sprint(
        &self,
        buffer: *mut ::std::os::raw::c_char,
        buffer_len: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_uri_sprint(self.get_inner(), buffer.into(), buffer_len.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn sprint_as_string(&self, max_length: usize) -> Result<String, AeronCError> {
        let mut result = String::with_capacity(max_length);
        self.sprint_into(&mut result)?;
        Ok(result)
    }
    #[inline]
    #[doc = "NOTE: allocation friendly method, the string capacity must be set as it will truncate string to capacity it will never grow the string. So if you pass String::new() it will write 0 chars"]
    pub fn sprint_into(&self, dst_truncate_to_capacity: &mut String) -> Result<i32, AeronCError> {
        unsafe {
            let capacity = dst_truncate_to_capacity.capacity();
            let vec = dst_truncate_to_capacity.as_mut_vec();
            vec.set_len(capacity);
            let result = self.sprint(vec.as_mut_ptr() as *mut _, capacity)?;
            let mut len = 0;
            loop {
                if len == capacity {
                    break;
                }
                let val = vec[len];
                if val == 0 {
                    break;
                }
                len += 1;
            }
            vec.set_len(len);
            Ok(result)
        }
    }
    #[inline]
    pub fn parse_tag(tag_str: &std::ffi::CStr) -> i64 {
        unsafe {
            let result = aeron_uri_parse_tag(tag_str.as_ptr());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_uri_t {
        self.inner.get()
    }
    #[inline(always)]
    pub fn get_inner_mut(&self) -> &mut aeron_uri_t {
        unsafe { &mut *self.inner.get() }
    }
    #[inline(always)]
    pub fn get_inner_ref(&self) -> &aeron_uri_t {
        unsafe { &*self.inner.get() }
    }
}
impl std::ops::Deref for AeronUri {
    type Target = aeron_uri_t;
    fn deref(&self) -> &Self::Target {
        self.get_inner_ref()
    }
}
impl From<*mut aeron_uri_t> for AeronUri {
    #[inline]
    fn from(value: *mut aeron_uri_t) -> Self {
        AeronUri {
            inner: CResource::Borrowed(value),
        }
    }
}
impl From<AeronUri> for *mut aeron_uri_t {
    #[inline]
    fn from(value: AeronUri) -> Self {
        value.get_inner()
    }
}
impl From<&AeronUri> for *mut aeron_uri_t {
    #[inline]
    fn from(value: &AeronUri) -> Self {
        value.get_inner()
    }
}
impl From<AeronUri> for aeron_uri_t {
    #[inline]
    fn from(value: AeronUri) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_uri_t> for AeronUri {
    #[inline]
    fn from(value: *const aeron_uri_t) -> Self {
        AeronUri {
            inner: CResource::Borrowed(value as *mut aeron_uri_t),
        }
    }
}
impl From<aeron_uri_t> for AeronUri {
    #[inline]
    fn from(value: aeron_uri_t) -> Self {
        AeronUri {
            inner: CResource::OwnedOnStack(MaybeUninit::new(value)),
        }
    }
}
impl Drop for AeronUri {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.as_owned() {
            if (inner.cleanup.is_none())
                && std::rc::Rc::strong_count(inner) == 1
                && !inner.is_closed_already_called()
            {
                if inner.auto_close.get() {
                    log::info!("auto closing {}", stringify!(AeronUri));
                    let result = self.close();
                    log::debug!("result {:?}", result);
                } else {
                    #[cfg(feature = "extra-logging")]
                    log::warn!("{} not closed", stringify!(AeronUri));
                }
            }
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronUri {
    fn default() -> Self {
        AeronUri::new_zeroed_on_heap()
    }
}
impl AeronUri {
    #[doc = r" Regular clone just increases the reference count of underlying count."]
    #[doc = r" `clone_struct` shallow copies the content of the underlying struct on heap."]
    #[doc = r""]
    #[doc = r" NOTE: if the struct has references to other structs these will not be copied"]
    #[doc = r""]
    #[doc = r" Must be only used on structs which has no init/clean up methods."]
    #[doc = r" So its danagerous to use with Aeron/AeronContext/AeronPublication/AeronSubscription"]
    #[doc = r" More intended for AeronArchiveRecordingDescriptor"]
    pub fn clone_struct(&self) -> Self {
        let copy = Self::default();
        copy.get_inner_mut().clone_from(self.deref());
        copy
    }
}
#[doc = "The error handler to be called when an error occurs."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronErrorHandlerCallback {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &str) -> ();
}
pub struct AeronErrorHandlerLogger;
impl AeronErrorHandlerCallback for AeronErrorHandlerLogger {
    fn handle_aeron_error_handler(&mut self, errcode: ::std::os::raw::c_int, message: &str) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_error_handler),
            [
                format!("{} : {:?}", stringify!(errcode), errcode),
                format!("{} : {:?}", stringify!(message), message)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronErrorHandlerLogger {}
unsafe impl Sync for AeronErrorHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_error_handler_handler() -> Option<&'static Handler<AeronErrorHandlerLogger>> {
        None::<&Handler<AeronErrorHandlerLogger>>
    }
}
#[allow(dead_code)]
#[doc = "The error handler to be called when an error occurs."]
unsafe extern "C" fn aeron_error_handler_t_callback<F: AeronErrorHandlerCallback>(
    clientd: *mut ::std::os::raw::c_void,
    errcode: ::std::os::raw::c_int,
    message: *const ::std::os::raw::c_char,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_error_handler));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_error_handler(
        errcode.into(),
        if message.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(message).to_str().unwrap() }
        },
    )
}
#[allow(dead_code)]
#[doc = "The error handler to be called when an error occurs."]
unsafe extern "C" fn aeron_error_handler_t_callback_for_once_closure<
    F: FnMut(::std::os::raw::c_int, &str) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    errcode: ::std::os::raw::c_int,
    message: *const ::std::os::raw::c_char,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_error_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        errcode.into(),
        if message.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(message).to_str().unwrap() }
        },
    )
}
#[doc = "The error frame handler to be called when the driver notifies the client about an error frame being received."]
#[doc = " The data passed to this callback will only be valid for the lifetime of the callback. The user should use"]
#[doc = " <code>aeron_publication_error_values_copy</code> if they require the data to live longer than that."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronPublicationErrorFrameHandlerCallback {
    fn handle_aeron_publication_error_frame_handler(
        &mut self,
        error_frame: AeronPublicationErrorValues,
    ) -> ();
}
pub struct AeronPublicationErrorFrameHandlerLogger;
impl AeronPublicationErrorFrameHandlerCallback for AeronPublicationErrorFrameHandlerLogger {
    fn handle_aeron_publication_error_frame_handler(
        &mut self,
        error_frame: AeronPublicationErrorValues,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_publication_error_frame_handler),
            [format!("{} : {:?}", stringify!(error_frame), error_frame)].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronPublicationErrorFrameHandlerLogger {}
unsafe impl Sync for AeronPublicationErrorFrameHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_publication_error_frame_handler_handler(
    ) -> Option<&'static Handler<AeronPublicationErrorFrameHandlerLogger>> {
        None::<&Handler<AeronPublicationErrorFrameHandlerLogger>>
    }
}
#[allow(dead_code)]
#[doc = "The error frame handler to be called when the driver notifies the client about an error frame being received."]
#[doc = " The data passed to this callback will only be valid for the lifetime of the callback. The user should use"]
#[doc = " <code>aeron_publication_error_values_copy</code> if they require the data to live longer than that."]
unsafe extern "C" fn aeron_publication_error_frame_handler_t_callback<
    F: AeronPublicationErrorFrameHandlerCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
    error_frame: *mut aeron_publication_error_values_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_publication_error_frame_handler)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_publication_error_frame_handler(error_frame.into())
}
#[allow(dead_code)]
#[doc = "The error frame handler to be called when the driver notifies the client about an error frame being received."]
#[doc = " The data passed to this callback will only be valid for the lifetime of the callback. The user should use"]
#[doc = " <code>aeron_publication_error_values_copy</code> if they require the data to live longer than that."]
unsafe extern "C" fn aeron_publication_error_frame_handler_t_callback_for_once_closure<
    F: FnMut(AeronPublicationErrorValues) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    error_frame: *mut aeron_publication_error_values_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_publication_error_frame_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(error_frame.into())
}
#[doc = "Generalised notification callback."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronNotificationCallback {
    fn handle_aeron_notification(&mut self) -> ();
}
pub struct AeronNotificationLogger;
impl AeronNotificationCallback for AeronNotificationLogger {
    fn handle_aeron_notification(&mut self) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_notification),
            [""].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronNotificationLogger {}
unsafe impl Sync for AeronNotificationLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_notification_handler() -> Option<&'static Handler<AeronNotificationLogger>> {
        None::<&Handler<AeronNotificationLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Generalised notification callback."]
unsafe extern "C" fn aeron_notification_t_callback<F: AeronNotificationCallback>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_notification));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_notification()
}
#[allow(dead_code)]
#[doc = "Generalised notification callback."]
unsafe extern "C" fn aeron_notification_t_callback_for_once_closure<F: FnMut() -> ()>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_notification_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure()
}
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_publication_t"]
#[doc = " or aeron_exclusive_publication_t successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for passing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original add publication call"]
#[doc = " @param channel of the publication"]
#[doc = " @param stream_id within the channel of the publication"]
#[doc = " @param session_id of the publication"]
#[doc = " @param correlation_id used by the publication"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronNewPublicationCallback {
    fn handle_aeron_on_new_publication(
        &mut self,
        async_: AeronAsyncAddPublication,
        channel: &str,
        stream_id: i32,
        session_id: i32,
        correlation_id: i64,
    ) -> ();
}
pub struct AeronNewPublicationLogger;
impl AeronNewPublicationCallback for AeronNewPublicationLogger {
    fn handle_aeron_on_new_publication(
        &mut self,
        async_: AeronAsyncAddPublication,
        channel: &str,
        stream_id: i32,
        session_id: i32,
        correlation_id: i64,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_new_publication),
            [
                format!("{} : {:?}", stringify!(async_), async_),
                format!("{} : {:?}", stringify!(channel), channel),
                format!("{} : {:?}", stringify!(stream_id), stream_id),
                format!("{} : {:?}", stringify!(session_id), session_id),
                format!("{} : {:?}", stringify!(correlation_id), correlation_id)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronNewPublicationLogger {}
unsafe impl Sync for AeronNewPublicationLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_new_publication_handler() -> Option<&'static Handler<AeronNewPublicationLogger>> {
        None::<&Handler<AeronNewPublicationLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_publication_t"]
#[doc = " or aeron_exclusive_publication_t successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for passing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original add publication call"]
#[doc = " @param channel of the publication"]
#[doc = " @param stream_id within the channel of the publication"]
#[doc = " @param session_id of the publication"]
#[doc = " @param correlation_id used by the publication"]
unsafe extern "C" fn aeron_on_new_publication_t_callback<F: AeronNewPublicationCallback>(
    clientd: *mut ::std::os::raw::c_void,
    async_: *mut aeron_async_add_publication_t,
    channel: *const ::std::os::raw::c_char,
    stream_id: i32,
    session_id: i32,
    correlation_id: i64,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_new_publication));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_new_publication(
        async_.into(),
        if channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(channel).to_str().unwrap() }
        },
        stream_id.into(),
        session_id.into(),
        correlation_id.into(),
    )
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_publication_t"]
#[doc = " or aeron_exclusive_publication_t successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for passing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original add publication call"]
#[doc = " @param channel of the publication"]
#[doc = " @param stream_id within the channel of the publication"]
#[doc = " @param session_id of the publication"]
#[doc = " @param correlation_id used by the publication"]
unsafe extern "C" fn aeron_on_new_publication_t_callback_for_once_closure<
    F: FnMut(AeronAsyncAddPublication, &str, i32, i32, i64) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    async_: *mut aeron_async_add_publication_t,
    channel: *const ::std::os::raw::c_char,
    stream_id: i32,
    session_id: i32,
    correlation_id: i64,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_new_publication_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        async_.into(),
        if channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(channel).to_str().unwrap() }
        },
        stream_id.into(),
        session_id.into(),
        correlation_id.into(),
    )
}
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_subscription_t"]
#[doc = " successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for handing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original aeron_add_async_subscription call"]
#[doc = " @param channel of the subscription"]
#[doc = " @param stream_id within the channel of the subscription"]
#[doc = " @param session_id of the subscription"]
#[doc = " @param correlation_id used by the subscription"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronNewSubscriptionCallback {
    fn handle_aeron_on_new_subscription(
        &mut self,
        async_: AeronAsyncAddSubscription,
        channel: &str,
        stream_id: i32,
        correlation_id: i64,
    ) -> ();
}
pub struct AeronNewSubscriptionLogger;
impl AeronNewSubscriptionCallback for AeronNewSubscriptionLogger {
    fn handle_aeron_on_new_subscription(
        &mut self,
        async_: AeronAsyncAddSubscription,
        channel: &str,
        stream_id: i32,
        correlation_id: i64,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_new_subscription),
            [
                format!("{} : {:?}", stringify!(async_), async_),
                format!("{} : {:?}", stringify!(channel), channel),
                format!("{} : {:?}", stringify!(stream_id), stream_id),
                format!("{} : {:?}", stringify!(correlation_id), correlation_id)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronNewSubscriptionLogger {}
unsafe impl Sync for AeronNewSubscriptionLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_new_subscription_handler() -> Option<&'static Handler<AeronNewSubscriptionLogger>> {
        None::<&Handler<AeronNewSubscriptionLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_subscription_t"]
#[doc = " successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for handing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original aeron_add_async_subscription call"]
#[doc = " @param channel of the subscription"]
#[doc = " @param stream_id within the channel of the subscription"]
#[doc = " @param session_id of the subscription"]
#[doc = " @param correlation_id used by the subscription"]
unsafe extern "C" fn aeron_on_new_subscription_t_callback<F: AeronNewSubscriptionCallback>(
    clientd: *mut ::std::os::raw::c_void,
    async_: *mut aeron_async_add_subscription_t,
    channel: *const ::std::os::raw::c_char,
    stream_id: i32,
    correlation_id: i64,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_new_subscription));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_new_subscription(
        async_.into(),
        if channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(channel).to_str().unwrap() }
        },
        stream_id.into(),
        correlation_id.into(),
    )
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notification that the media driver has added an aeron_subscription_t"]
#[doc = " successfully."]
#[doc = ""]
#[doc = " Implementations should do the minimum work for handing off state to another thread for later processing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call"]
#[doc = " @param async associated with the original aeron_add_async_subscription call"]
#[doc = " @param channel of the subscription"]
#[doc = " @param stream_id within the channel of the subscription"]
#[doc = " @param session_id of the subscription"]
#[doc = " @param correlation_id used by the subscription"]
unsafe extern "C" fn aeron_on_new_subscription_t_callback_for_once_closure<
    F: FnMut(AeronAsyncAddSubscription, &str, i32, i64) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    async_: *mut aeron_async_add_subscription_t,
    channel: *const ::std::os::raw::c_char,
    stream_id: i32,
    correlation_id: i64,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_new_subscription_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        async_.into(),
        if channel.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(channel).to_str().unwrap() }
        },
        stream_id.into(),
        correlation_id.into(),
    )
}
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t was added."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become available."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronAvailableImageCallback {
    fn handle_aeron_on_available_image(
        &mut self,
        subscription: AeronSubscription,
        image: AeronImage,
    ) -> ();
}
pub struct AeronAvailableImageLogger;
impl AeronAvailableImageCallback for AeronAvailableImageLogger {
    fn handle_aeron_on_available_image(
        &mut self,
        subscription: AeronSubscription,
        image: AeronImage,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_available_image),
            [
                format!("{} : {:?}", stringify!(subscription), subscription),
                format!("{} : {:?}", stringify!(image), image)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronAvailableImageLogger {}
unsafe impl Sync for AeronAvailableImageLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_available_image_handler() -> Option<&'static Handler<AeronAvailableImageLogger>> {
        None::<&Handler<AeronAvailableImageLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t was added."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become available."]
unsafe extern "C" fn aeron_on_available_image_t_callback<F: AeronAvailableImageCallback>(
    clientd: *mut ::std::os::raw::c_void,
    subscription: *mut aeron_subscription_t,
    image: *mut aeron_image_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_available_image));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_available_image(subscription.into(), image.into())
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t was added."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become available."]
unsafe extern "C" fn aeron_on_available_image_t_callback_for_once_closure<
    F: FnMut(AeronSubscription, AeronImage) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    subscription: *mut aeron_subscription_t,
    image: *mut aeron_image_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_available_image_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(subscription.into(), image.into())
}
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t has been removed from use and"]
#[doc = " should not be used any longer."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become unavailable."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronUnavailableImageCallback {
    fn handle_aeron_on_unavailable_image(
        &mut self,
        subscription: AeronSubscription,
        image: AeronImage,
    ) -> ();
}
pub struct AeronUnavailableImageLogger;
impl AeronUnavailableImageCallback for AeronUnavailableImageLogger {
    fn handle_aeron_on_unavailable_image(
        &mut self,
        subscription: AeronSubscription,
        image: AeronImage,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_unavailable_image),
            [
                format!("{} : {:?}", stringify!(subscription), subscription),
                format!("{} : {:?}", stringify!(image), image)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronUnavailableImageLogger {}
unsafe impl Sync for AeronUnavailableImageLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_unavailable_image_handler() -> Option<&'static Handler<AeronUnavailableImageLogger>> {
        None::<&Handler<AeronUnavailableImageLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t has been removed from use and"]
#[doc = " should not be used any longer."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become unavailable."]
unsafe extern "C" fn aeron_on_unavailable_image_t_callback<F: AeronUnavailableImageCallback>(
    clientd: *mut ::std::os::raw::c_void,
    subscription: *mut aeron_subscription_t,
    image: *mut aeron_image_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_unavailable_image));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_unavailable_image(subscription.into(), image.into())
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that an aeron_image_t has been removed from use and"]
#[doc = " should not be used any longer."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param subscription that image is part of."]
#[doc = " @param image that has become unavailable."]
unsafe extern "C" fn aeron_on_unavailable_image_t_callback_for_once_closure<
    F: FnMut(AeronSubscription, AeronImage) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    subscription: *mut aeron_subscription_t,
    image: *mut aeron_image_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_unavailable_image_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(subscription.into(), image.into())
}
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been added to the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronAvailableCounterCallback {
    fn handle_aeron_on_available_counter(
        &mut self,
        counters_reader: AeronCountersReader,
        registration_id: i64,
        counter_id: i32,
    ) -> ();
}
pub struct AeronAvailableCounterLogger;
impl AeronAvailableCounterCallback for AeronAvailableCounterLogger {
    fn handle_aeron_on_available_counter(
        &mut self,
        counters_reader: AeronCountersReader,
        registration_id: i64,
        counter_id: i32,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_available_counter),
            [
                format!("{} : {:?}", stringify!(counters_reader), counters_reader),
                format!("{} : {:?}", stringify!(registration_id), registration_id),
                format!("{} : {:?}", stringify!(counter_id), counter_id)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronAvailableCounterLogger {}
unsafe impl Sync for AeronAvailableCounterLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_available_counter_handler() -> Option<&'static Handler<AeronAvailableCounterLogger>> {
        None::<&Handler<AeronAvailableCounterLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been added to the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
unsafe extern "C" fn aeron_on_available_counter_t_callback<F: AeronAvailableCounterCallback>(
    clientd: *mut ::std::os::raw::c_void,
    counters_reader: *mut aeron_counters_reader_t,
    registration_id: i64,
    counter_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_available_counter));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_available_counter(
        counters_reader.into(),
        registration_id.into(),
        counter_id.into(),
    )
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been added to the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
unsafe extern "C" fn aeron_on_available_counter_t_callback_for_once_closure<
    F: FnMut(AeronCountersReader, i64, i32) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    counters_reader: *mut aeron_counters_reader_t,
    registration_id: i64,
    counter_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_available_counter_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        counters_reader.into(),
        registration_id.into(),
        counter_id.into(),
    )
}
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been removed from the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronUnavailableCounterCallback {
    fn handle_aeron_on_unavailable_counter(
        &mut self,
        counters_reader: AeronCountersReader,
        registration_id: i64,
        counter_id: i32,
    ) -> ();
}
pub struct AeronUnavailableCounterLogger;
impl AeronUnavailableCounterCallback for AeronUnavailableCounterLogger {
    fn handle_aeron_on_unavailable_counter(
        &mut self,
        counters_reader: AeronCountersReader,
        registration_id: i64,
        counter_id: i32,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_unavailable_counter),
            [
                format!("{} : {:?}", stringify!(counters_reader), counters_reader),
                format!("{} : {:?}", stringify!(registration_id), registration_id),
                format!("{} : {:?}", stringify!(counter_id), counter_id)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronUnavailableCounterLogger {}
unsafe impl Sync for AeronUnavailableCounterLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_unavailable_counter_handler(
    ) -> Option<&'static Handler<AeronUnavailableCounterLogger>> {
        None::<&Handler<AeronUnavailableCounterLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been removed from the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
unsafe extern "C" fn aeron_on_unavailable_counter_t_callback<F: AeronUnavailableCounterCallback>(
    clientd: *mut ::std::os::raw::c_void,
    counters_reader: *mut aeron_counters_reader_t,
    registration_id: i64,
    counter_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_on_unavailable_counter)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_unavailable_counter(
        counters_reader.into(),
        registration_id.into(),
        counter_id.into(),
    )
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that a counter has been removed from the driver."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = " @param counters_reader that holds the counter."]
#[doc = " @param registration_id of the counter."]
#[doc = " @param counter_id of the counter."]
unsafe extern "C" fn aeron_on_unavailable_counter_t_callback_for_once_closure<
    F: FnMut(AeronCountersReader, i64, i32) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    counters_reader: *mut aeron_counters_reader_t,
    registration_id: i64,
    counter_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_unavailable_counter_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        counters_reader.into(),
        registration_id.into(),
        counter_id.into(),
    )
}
#[doc = "Function called by aeron_client_t to deliver notifications that the client is closing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronCloseClientCallback {
    fn handle_aeron_on_close_client(&mut self) -> ();
}
pub struct AeronCloseClientLogger;
impl AeronCloseClientCallback for AeronCloseClientLogger {
    fn handle_aeron_on_close_client(&mut self) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_on_close_client),
            [""].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronCloseClientLogger {}
unsafe impl Sync for AeronCloseClientLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_close_client_handler() -> Option<&'static Handler<AeronCloseClientLogger>> {
        None::<&Handler<AeronCloseClientLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that the client is closing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
unsafe extern "C" fn aeron_on_close_client_t_callback<F: AeronCloseClientCallback>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_on_close_client));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_on_close_client()
}
#[allow(dead_code)]
#[doc = "Function called by aeron_client_t to deliver notifications that the client is closing."]
#[doc = ""]
#[doc = " @param clientd to be returned in the call."]
unsafe extern "C" fn aeron_on_close_client_t_callback_for_once_closure<F: FnMut() -> ()>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_on_close_client_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure()
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronAgentStartFuncCallback {
    fn handle_aeron_agent_on_start_func(&mut self, role_name: &str) -> ();
}
pub struct AeronAgentStartFuncLogger;
impl AeronAgentStartFuncCallback for AeronAgentStartFuncLogger {
    fn handle_aeron_agent_on_start_func(&mut self, role_name: &str) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_agent_on_start_func),
            [format!("{} : {:?}", stringify!(role_name), role_name)].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronAgentStartFuncLogger {}
unsafe impl Sync for AeronAgentStartFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_agent_start_func_handler() -> Option<&'static Handler<AeronAgentStartFuncLogger>> {
        None::<&Handler<AeronAgentStartFuncLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_agent_on_start_func_t_callback<F: AeronAgentStartFuncCallback>(
    state: *mut ::std::os::raw::c_void,
    role_name: *const ::std::os::raw::c_char,
) -> () {
    #[cfg(debug_assertions)]
    if state.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_agent_on_start_func));
    }
    let closure: &mut F = &mut *(state as *mut F);
    closure.handle_aeron_agent_on_start_func(if role_name.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(role_name).to_str().unwrap() }
    })
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_agent_on_start_func_t_callback_for_once_closure<F: FnMut(&str) -> ()>(
    state: *mut ::std::os::raw::c_void,
    role_name: *const ::std::os::raw::c_char,
) -> () {
    #[cfg(debug_assertions)]
    if state.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_agent_on_start_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(state as *mut F);
    closure(if role_name.is_null() {
        ""
    } else {
        unsafe { std::ffi::CStr::from_ptr(role_name).to_str().unwrap() }
    })
}
#[doc = "Function called by aeron_counters_reader_foreach_counter for each counter in the aeron_counters_reader_t."]
#[doc = ""]
#[doc = " @param value of the counter."]
#[doc = " @param id of the counter."]
#[doc = " @param label for the counter."]
#[doc = " @param label_length for the counter."]
#[doc = " @param clientd to be returned in the call"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronCountersReaderForeachCounterFuncCallback {
    fn handle_aeron_counters_reader_foreach_counter_func(
        &mut self,
        value: i64,
        id: i32,
        type_id: i32,
        key: &[u8],
        label: &str,
    ) -> ();
}
pub struct AeronCountersReaderForeachCounterFuncLogger;
impl AeronCountersReaderForeachCounterFuncCallback for AeronCountersReaderForeachCounterFuncLogger {
    fn handle_aeron_counters_reader_foreach_counter_func(
        &mut self,
        value: i64,
        id: i32,
        type_id: i32,
        key: &[u8],
        label: &str,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_counters_reader_foreach_counter_func),
            [
                format!("{} : {:?}", stringify!(value), value),
                format!("{} : {:?}", stringify!(id), id),
                format!("{} : {:?}", stringify!(type_id), type_id),
                format!("{} : {:?}", stringify!(key), key),
                format!("{} : {:?}", stringify!(label), label)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronCountersReaderForeachCounterFuncLogger {}
unsafe impl Sync for AeronCountersReaderForeachCounterFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_counters_reader_foreach_counter_func_handler(
    ) -> Option<&'static Handler<AeronCountersReaderForeachCounterFuncLogger>> {
        None::<&Handler<AeronCountersReaderForeachCounterFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called by aeron_counters_reader_foreach_counter for each counter in the aeron_counters_reader_t."]
#[doc = ""]
#[doc = " @param value of the counter."]
#[doc = " @param id of the counter."]
#[doc = " @param label for the counter."]
#[doc = " @param label_length for the counter."]
#[doc = " @param clientd to be returned in the call"]
unsafe extern "C" fn aeron_counters_reader_foreach_counter_func_t_callback<
    F: AeronCountersReaderForeachCounterFuncCallback,
>(
    value: i64,
    id: i32,
    type_id: i32,
    key: *const u8,
    key_length: usize,
    label: *const ::std::os::raw::c_char,
    label_length: usize,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_counters_reader_foreach_counter_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_counters_reader_foreach_counter_func(
        value.into(),
        id.into(),
        type_id.into(),
        if key.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(key, key_length)
        },
        if label.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                label as *const u8,
                label_length.try_into().unwrap(),
            ))
        },
    )
}
#[allow(dead_code)]
#[doc = "Function called by aeron_counters_reader_foreach_counter for each counter in the aeron_counters_reader_t."]
#[doc = ""]
#[doc = " @param value of the counter."]
#[doc = " @param id of the counter."]
#[doc = " @param label for the counter."]
#[doc = " @param label_length for the counter."]
#[doc = " @param clientd to be returned in the call"]
unsafe extern "C" fn aeron_counters_reader_foreach_counter_func_t_callback_for_once_closure<
    F: FnMut(i64, i32, i32, &[u8], &str) -> (),
>(
    value: i64,
    id: i32,
    type_id: i32,
    key: *const u8,
    key_length: usize,
    label: *const ::std::os::raw::c_char,
    label_length: usize,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_counters_reader_foreach_counter_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        value.into(),
        id.into(),
        type_id.into(),
        if key.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(key, key_length)
        },
        if label.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                label as *const u8,
                label_length.try_into().unwrap(),
            ))
        },
    )
}
#[doc = "Function called when filling in the reserved value field of a message."]
#[doc = ""]
#[doc = " @param clientd passed to the offer function."]
#[doc = " @param buffer of the entire frame, including Aeron data header."]
#[doc = " @param frame_length of the entire frame."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronReservedValueSupplierCallback {
    fn handle_aeron_reserved_value_supplier(&mut self, buffer: *mut u8, frame_length: usize)
        -> i64;
}
pub struct AeronReservedValueSupplierLogger;
impl AeronReservedValueSupplierCallback for AeronReservedValueSupplierLogger {
    fn handle_aeron_reserved_value_supplier(
        &mut self,
        buffer: *mut u8,
        frame_length: usize,
    ) -> i64 {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_reserved_value_supplier),
            [
                format!("{} : {:?}", stringify!(buffer), buffer),
                format!("{} : {:?}", stringify!(frame_length), frame_length)
            ]
            .join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronReservedValueSupplierLogger {}
unsafe impl Sync for AeronReservedValueSupplierLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_reserved_value_supplier_handler(
    ) -> Option<&'static Handler<AeronReservedValueSupplierLogger>> {
        None::<&Handler<AeronReservedValueSupplierLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Function called when filling in the reserved value field of a message."]
#[doc = ""]
#[doc = " @param clientd passed to the offer function."]
#[doc = " @param buffer of the entire frame, including Aeron data header."]
#[doc = " @param frame_length of the entire frame."]
unsafe extern "C" fn aeron_reserved_value_supplier_t_callback<
    F: AeronReservedValueSupplierCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *mut u8,
    frame_length: usize,
) -> i64 {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_reserved_value_supplier)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_reserved_value_supplier(buffer.into(), frame_length.into())
}
#[allow(dead_code)]
#[doc = "Function called when filling in the reserved value field of a message."]
#[doc = ""]
#[doc = " @param clientd passed to the offer function."]
#[doc = " @param buffer of the entire frame, including Aeron data header."]
#[doc = " @param frame_length of the entire frame."]
unsafe extern "C" fn aeron_reserved_value_supplier_t_callback_for_once_closure<
    F: FnMut(*mut u8, usize) -> i64,
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *mut u8,
    frame_length: usize,
) -> i64 {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_reserved_value_supplier_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(buffer.into(), frame_length.into())
}
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " The frame will either contain a whole message or a fragment of a message to be reassembled. Messages are fragmented"]
#[doc = " if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronFragmentHandlerCallback {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], header: AeronHeader) -> ();
}
pub struct AeronFragmentHandlerLogger;
impl AeronFragmentHandlerCallback for AeronFragmentHandlerLogger {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], header: AeronHeader) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_fragment_handler),
            [
                format!("{} : {:?}", stringify!(buffer), buffer),
                format!("{} : {:?}", stringify!(header), header)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronFragmentHandlerLogger {}
unsafe impl Sync for AeronFragmentHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_fragment_handler_handler() -> Option<&'static Handler<AeronFragmentHandlerLogger>> {
        None::<&Handler<AeronFragmentHandlerLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " The frame will either contain a whole message or a fragment of a message to be reassembled. Messages are fragmented"]
#[doc = " if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
unsafe extern "C" fn aeron_fragment_handler_t_callback<F: AeronFragmentHandlerCallback>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    header: *mut aeron_header_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_fragment_handler));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_fragment_handler(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        header.into(),
    )
}
#[allow(dead_code)]
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " The frame will either contain a whole message or a fragment of a message to be reassembled. Messages are fragmented"]
#[doc = " if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
unsafe extern "C" fn aeron_fragment_handler_t_callback_for_once_closure<
    F: FnMut(&[u8], AeronHeader) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    header: *mut aeron_header_t,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_fragment_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        header.into(),
    )
}
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " Handler for reading data that is coming from a log buffer. The frame will either contain a whole message"]
#[doc = " or a fragment of a message to be reassembled. Messages are fragmented if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the controlled poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
#[doc = " @return The action to be taken with regard to the stream position after the callback."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronControlledFragmentHandlerCallback {
    fn handle_aeron_controlled_fragment_handler(
        &mut self,
        buffer: &[u8],
        header: AeronHeader,
    ) -> aeron_controlled_fragment_handler_action_t;
}
pub struct AeronControlledFragmentHandlerLogger;
impl AeronControlledFragmentHandlerCallback for AeronControlledFragmentHandlerLogger {
    fn handle_aeron_controlled_fragment_handler(
        &mut self,
        buffer: &[u8],
        header: AeronHeader,
    ) -> aeron_controlled_fragment_handler_action_t {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_controlled_fragment_handler),
            [
                format!("{} : {:?}", stringify!(buffer), buffer),
                format!("{} : {:?}", stringify!(header), header)
            ]
            .join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronControlledFragmentHandlerLogger {}
unsafe impl Sync for AeronControlledFragmentHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_controlled_fragment_handler_handler(
    ) -> Option<&'static Handler<AeronControlledFragmentHandlerLogger>> {
        None::<&Handler<AeronControlledFragmentHandlerLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " Handler for reading data that is coming from a log buffer. The frame will either contain a whole message"]
#[doc = " or a fragment of a message to be reassembled. Messages are fragmented if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the controlled poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
#[doc = " @return The action to be taken with regard to the stream position after the callback."]
unsafe extern "C" fn aeron_controlled_fragment_handler_t_callback<
    F: AeronControlledFragmentHandlerCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    header: *mut aeron_header_t,
) -> aeron_controlled_fragment_handler_action_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_controlled_fragment_handler)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_controlled_fragment_handler(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        header.into(),
    )
}
#[allow(dead_code)]
#[doc = "Callback for handling fragments of data being read from a log."]
#[doc = ""]
#[doc = " Handler for reading data that is coming from a log buffer. The frame will either contain a whole message"]
#[doc = " or a fragment of a message to be reassembled. Messages are fragmented if greater than the frame for MTU in length."]
#[doc = ""]
#[doc = " @param clientd passed to the controlled poll function."]
#[doc = " @param buffer containing the data."]
#[doc = " @param length of the data in bytes."]
#[doc = " @param header representing the meta data for the data."]
#[doc = " @return The action to be taken with regard to the stream position after the callback."]
unsafe extern "C" fn aeron_controlled_fragment_handler_t_callback_for_once_closure<
    F: FnMut(&[u8], AeronHeader) -> aeron_controlled_fragment_handler_action_t,
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    header: *mut aeron_header_t,
) -> aeron_controlled_fragment_handler_action_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_controlled_fragment_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        header.into(),
    )
}
#[doc = "Callback for handling a block of messages being read from a log."]
#[doc = ""]
#[doc = " @param clientd passed to the block poll function."]
#[doc = " @param buffer containing the block of message fragments."]
#[doc = " @param offset at which the block begins, including any frame headers."]
#[doc = " @param length of the block in bytes, including any frame headers that is aligned."]
#[doc = " @param session_id of the stream containing this block of message fragments."]
#[doc = " @param term_id of the stream containing this block of message fragments."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronBlockHandlerCallback {
    fn handle_aeron_block_handler(&mut self, buffer: &[u8], session_id: i32, term_id: i32) -> ();
}
pub struct AeronBlockHandlerLogger;
impl AeronBlockHandlerCallback for AeronBlockHandlerLogger {
    fn handle_aeron_block_handler(&mut self, buffer: &[u8], session_id: i32, term_id: i32) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_block_handler),
            [
                format!("{} : {:?}", stringify!(buffer), buffer),
                format!("{} : {:?}", stringify!(session_id), session_id),
                format!("{} : {:?}", stringify!(term_id), term_id)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronBlockHandlerLogger {}
unsafe impl Sync for AeronBlockHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_block_handler_handler() -> Option<&'static Handler<AeronBlockHandlerLogger>> {
        None::<&Handler<AeronBlockHandlerLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback for handling a block of messages being read from a log."]
#[doc = ""]
#[doc = " @param clientd passed to the block poll function."]
#[doc = " @param buffer containing the block of message fragments."]
#[doc = " @param offset at which the block begins, including any frame headers."]
#[doc = " @param length of the block in bytes, including any frame headers that is aligned."]
#[doc = " @param session_id of the stream containing this block of message fragments."]
#[doc = " @param term_id of the stream containing this block of message fragments."]
unsafe extern "C" fn aeron_block_handler_t_callback<F: AeronBlockHandlerCallback>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    session_id: i32,
    term_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_block_handler));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_block_handler(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        session_id.into(),
        term_id.into(),
    )
}
#[allow(dead_code)]
#[doc = "Callback for handling a block of messages being read from a log."]
#[doc = ""]
#[doc = " @param clientd passed to the block poll function."]
#[doc = " @param buffer containing the block of message fragments."]
#[doc = " @param offset at which the block begins, including any frame headers."]
#[doc = " @param length of the block in bytes, including any frame headers that is aligned."]
#[doc = " @param session_id of the stream containing this block of message fragments."]
#[doc = " @param term_id of the stream containing this block of message fragments."]
unsafe extern "C" fn aeron_block_handler_t_callback_for_once_closure<
    F: FnMut(&[u8], i32, i32) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    buffer: *const u8,
    length: usize,
    session_id: i32,
    term_id: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_block_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        if buffer.is_null() {
            &[] as &[_]
        } else {
            std::slice::from_raw_parts(buffer, length)
        },
        session_id.into(),
        term_id.into(),
    )
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronErrorLogReaderFuncCallback {
    fn handle_aeron_error_log_reader_func(
        &mut self,
        observation_count: i32,
        first_observation_timestamp: i64,
        last_observation_timestamp: i64,
        error: &str,
    ) -> ();
}
pub struct AeronErrorLogReaderFuncLogger;
impl AeronErrorLogReaderFuncCallback for AeronErrorLogReaderFuncLogger {
    fn handle_aeron_error_log_reader_func(
        &mut self,
        observation_count: i32,
        first_observation_timestamp: i64,
        last_observation_timestamp: i64,
        error: &str,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_error_log_reader_func),
            [
                format!(
                    "{} : {:?}",
                    stringify!(observation_count),
                    observation_count
                ),
                format!(
                    "{} : {:?}",
                    stringify!(first_observation_timestamp),
                    first_observation_timestamp
                ),
                format!(
                    "{} : {:?}",
                    stringify!(last_observation_timestamp),
                    last_observation_timestamp
                ),
                format!("{} : {:?}", stringify!(error), error)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronErrorLogReaderFuncLogger {}
unsafe impl Sync for AeronErrorLogReaderFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_error_log_reader_func_handler(
    ) -> Option<&'static Handler<AeronErrorLogReaderFuncLogger>> {
        None::<&Handler<AeronErrorLogReaderFuncLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_error_log_reader_func_t_callback<F: AeronErrorLogReaderFuncCallback>(
    observation_count: i32,
    first_observation_timestamp: i64,
    last_observation_timestamp: i64,
    error: *const ::std::os::raw::c_char,
    error_length: usize,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_error_log_reader_func));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_error_log_reader_func(
        observation_count.into(),
        first_observation_timestamp.into(),
        last_observation_timestamp.into(),
        if error.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                error as *const u8,
                error_length.try_into().unwrap(),
            ))
        },
    )
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_error_log_reader_func_t_callback_for_once_closure<
    F: FnMut(i32, i64, i64, &str) -> (),
>(
    observation_count: i32,
    first_observation_timestamp: i64,
    last_observation_timestamp: i64,
    error: *const ::std::os::raw::c_char,
    error_length: usize,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_error_log_reader_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        observation_count.into(),
        first_observation_timestamp.into(),
        last_observation_timestamp.into(),
        if error.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                error as *const u8,
                error_length.try_into().unwrap(),
            ))
        },
    )
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronLossReporterReadEntryFuncCallback {
    fn handle_aeron_loss_reporter_read_entry_func(
        &mut self,
        observation_count: i64,
        total_bytes_lost: i64,
        first_observation_timestamp: i64,
        last_observation_timestamp: i64,
        session_id: i32,
        stream_id: i32,
        channel: &str,
        source: &str,
    ) -> ();
}
pub struct AeronLossReporterReadEntryFuncLogger;
impl AeronLossReporterReadEntryFuncCallback for AeronLossReporterReadEntryFuncLogger {
    fn handle_aeron_loss_reporter_read_entry_func(
        &mut self,
        observation_count: i64,
        total_bytes_lost: i64,
        first_observation_timestamp: i64,
        last_observation_timestamp: i64,
        session_id: i32,
        stream_id: i32,
        channel: &str,
        source: &str,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_loss_reporter_read_entry_func),
            [
                format!(
                    "{} : {:?}",
                    stringify!(observation_count),
                    observation_count
                ),
                format!("{} : {:?}", stringify!(total_bytes_lost), total_bytes_lost),
                format!(
                    "{} : {:?}",
                    stringify!(first_observation_timestamp),
                    first_observation_timestamp
                ),
                format!(
                    "{} : {:?}",
                    stringify!(last_observation_timestamp),
                    last_observation_timestamp
                ),
                format!("{} : {:?}", stringify!(session_id), session_id),
                format!("{} : {:?}", stringify!(stream_id), stream_id),
                format!("{} : {:?}", stringify!(channel), channel),
                format!("{} : {:?}", stringify!(source), source)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronLossReporterReadEntryFuncLogger {}
unsafe impl Sync for AeronLossReporterReadEntryFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_loss_reporter_read_entry_func_handler(
    ) -> Option<&'static Handler<AeronLossReporterReadEntryFuncLogger>> {
        None::<&Handler<AeronLossReporterReadEntryFuncLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_loss_reporter_read_entry_func_t_callback<
    F: AeronLossReporterReadEntryFuncCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
    observation_count: i64,
    total_bytes_lost: i64,
    first_observation_timestamp: i64,
    last_observation_timestamp: i64,
    session_id: i32,
    stream_id: i32,
    channel: *const ::std::os::raw::c_char,
    channel_length: i32,
    source: *const ::std::os::raw::c_char,
    source_length: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_loss_reporter_read_entry_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_loss_reporter_read_entry_func(
        observation_count.into(),
        total_bytes_lost.into(),
        first_observation_timestamp.into(),
        last_observation_timestamp.into(),
        session_id.into(),
        stream_id.into(),
        if channel.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                channel as *const u8,
                channel_length.try_into().unwrap(),
            ))
        },
        if source.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                source as *const u8,
                source_length.try_into().unwrap(),
            ))
        },
    )
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_loss_reporter_read_entry_func_t_callback_for_once_closure<
    F: FnMut(i64, i64, i64, i64, i32, i32, &str, &str) -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
    observation_count: i64,
    total_bytes_lost: i64,
    first_observation_timestamp: i64,
    last_observation_timestamp: i64,
    session_id: i32,
    stream_id: i32,
    channel: *const ::std::os::raw::c_char,
    channel_length: i32,
    source: *const ::std::os::raw::c_char,
    source_length: i32,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_loss_reporter_read_entry_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        observation_count.into(),
        total_bytes_lost.into(),
        first_observation_timestamp.into(),
        last_observation_timestamp.into(),
        session_id.into(),
        stream_id.into(),
        if channel.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                channel as *const u8,
                channel_length.try_into().unwrap(),
            ))
        },
        if source.is_null() {
            ""
        } else {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                source as *const u8,
                source_length.try_into().unwrap(),
            ))
        },
    )
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronIdleStrategyFuncCallback {
    fn handle_aeron_idle_strategy_func(&mut self, work_count: ::std::os::raw::c_int) -> ();
}
pub struct AeronIdleStrategyFuncLogger;
impl AeronIdleStrategyFuncCallback for AeronIdleStrategyFuncLogger {
    fn handle_aeron_idle_strategy_func(&mut self, work_count: ::std::os::raw::c_int) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_idle_strategy_func),
            [format!("{} : {:?}", stringify!(work_count), work_count)].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronIdleStrategyFuncLogger {}
unsafe impl Sync for AeronIdleStrategyFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_idle_strategy_func_handler() -> Option<&'static Handler<AeronIdleStrategyFuncLogger>>
    {
        None::<&Handler<AeronIdleStrategyFuncLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_idle_strategy_func_t_callback<F: AeronIdleStrategyFuncCallback>(
    state: *mut ::std::os::raw::c_void,
    work_count: ::std::os::raw::c_int,
) -> () {
    #[cfg(debug_assertions)]
    if state.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_idle_strategy_func));
    }
    let closure: &mut F = &mut *(state as *mut F);
    closure.handle_aeron_idle_strategy_func(work_count.into())
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_idle_strategy_func_t_callback_for_once_closure<
    F: FnMut(::std::os::raw::c_int) -> (),
>(
    state: *mut ::std::os::raw::c_void,
    work_count: ::std::os::raw::c_int,
) -> () {
    #[cfg(debug_assertions)]
    if state.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_idle_strategy_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(state as *mut F);
    closure(work_count.into())
}
#[doc = "Callback to return encoded credentials."]
#[doc = ""]
#[doc = " @return encoded credentials to include with the connect request"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveCredentialsEncodedCredentialsSupplierFuncCallback {
    fn handle_aeron_archive_credentials_encoded_credentials_supplier_func(
        &mut self,
    ) -> *mut aeron_archive_encoded_credentials_t;
}
pub struct AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger;
impl AeronArchiveCredentialsEncodedCredentialsSupplierFuncCallback
    for AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger
{
    fn handle_aeron_archive_credentials_encoded_credentials_supplier_func(
        &mut self,
    ) -> *mut aeron_archive_encoded_credentials_t {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_credentials_encoded_credentials_supplier_func),
            [""].join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger {}
unsafe impl Sync for AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_credentials_encoded_credentials_supplier_func_handler(
    ) -> Option<&'static Handler<AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger>> {
        None::<&Handler<AeronArchiveCredentialsEncodedCredentialsSupplierFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials."]
#[doc = ""]
#[doc = " @return encoded credentials to include with the connect request"]
unsafe extern "C" fn aeron_archive_credentials_encoded_credentials_supplier_func_t_callback<
    F: AeronArchiveCredentialsEncodedCredentialsSupplierFuncCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_credentials_encoded_credentials_supplier_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_credentials_encoded_credentials_supplier_func()
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials."]
#[doc = ""]
#[doc = " @return encoded credentials to include with the connect request"]
unsafe extern "C" fn aeron_archive_credentials_encoded_credentials_supplier_func_t_callback_for_once_closure<
    F: FnMut() -> *mut aeron_archive_encoded_credentials_t,
>(
    clientd: *mut ::std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log :: debug ! ("calling {}" , stringify ! (aeron_archive_credentials_encoded_credentials_supplier_func_t_callback_for_once_closure));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure()
}
#[doc = "Callback to return encoded credentials given a specific encoded challenge."]
#[doc = ""]
#[doc = " @param encoded_challenge to use to generate the encoded credentials"]
#[doc = " @return encoded credentials to include with the challenge response"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveCredentialsChallengeSupplierFuncCallback {
    fn handle_aeron_archive_credentials_challenge_supplier_func(
        &mut self,
        encoded_challenge: AeronArchiveEncodedCredentials,
    ) -> *mut aeron_archive_encoded_credentials_t;
}
pub struct AeronArchiveCredentialsChallengeSupplierFuncLogger;
impl AeronArchiveCredentialsChallengeSupplierFuncCallback
    for AeronArchiveCredentialsChallengeSupplierFuncLogger
{
    fn handle_aeron_archive_credentials_challenge_supplier_func(
        &mut self,
        encoded_challenge: AeronArchiveEncodedCredentials,
    ) -> *mut aeron_archive_encoded_credentials_t {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_credentials_challenge_supplier_func),
            [format!(
                "{} : {:?}",
                stringify!(encoded_challenge),
                encoded_challenge
            )]
            .join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronArchiveCredentialsChallengeSupplierFuncLogger {}
unsafe impl Sync for AeronArchiveCredentialsChallengeSupplierFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_credentials_challenge_supplier_func_handler(
    ) -> Option<&'static Handler<AeronArchiveCredentialsChallengeSupplierFuncLogger>> {
        None::<&Handler<AeronArchiveCredentialsChallengeSupplierFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials given a specific encoded challenge."]
#[doc = ""]
#[doc = " @param encoded_challenge to use to generate the encoded credentials"]
#[doc = " @return encoded credentials to include with the challenge response"]
unsafe extern "C" fn aeron_archive_credentials_challenge_supplier_func_t_callback<
    F: AeronArchiveCredentialsChallengeSupplierFuncCallback,
>(
    encoded_challenge: *mut aeron_archive_encoded_credentials_t,
    clientd: *mut ::std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_credentials_challenge_supplier_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_credentials_challenge_supplier_func(encoded_challenge.into())
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials given a specific encoded challenge."]
#[doc = ""]
#[doc = " @param encoded_challenge to use to generate the encoded credentials"]
#[doc = " @return encoded credentials to include with the challenge response"]
unsafe extern "C" fn aeron_archive_credentials_challenge_supplier_func_t_callback_for_once_closure<
    F: FnMut(AeronArchiveEncodedCredentials) -> *mut aeron_archive_encoded_credentials_t,
>(
    encoded_challenge: *mut aeron_archive_encoded_credentials_t,
    clientd: *mut ::std::os::raw::c_void,
) -> *mut aeron_archive_encoded_credentials_t {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(
                aeron_archive_credentials_challenge_supplier_func_t_callback_for_once_closure
            )
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(encoded_challenge.into())
}
#[doc = "Callback to return encoded credentials so they may be reused or freed."]
#[doc = ""]
#[doc = " @param credentials to reuse or free"]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveCredentialsFreeFuncCallback {
    fn handle_aeron_archive_credentials_free_func(
        &mut self,
        credentials: AeronArchiveEncodedCredentials,
    ) -> ();
}
pub struct AeronArchiveCredentialsFreeFuncLogger;
impl AeronArchiveCredentialsFreeFuncCallback for AeronArchiveCredentialsFreeFuncLogger {
    fn handle_aeron_archive_credentials_free_func(
        &mut self,
        credentials: AeronArchiveEncodedCredentials,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_credentials_free_func),
            [format!("{} : {:?}", stringify!(credentials), credentials)].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronArchiveCredentialsFreeFuncLogger {}
unsafe impl Sync for AeronArchiveCredentialsFreeFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_credentials_free_func_handler(
    ) -> Option<&'static Handler<AeronArchiveCredentialsFreeFuncLogger>> {
        None::<&Handler<AeronArchiveCredentialsFreeFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials so they may be reused or freed."]
#[doc = ""]
#[doc = " @param credentials to reuse or free"]
unsafe extern "C" fn aeron_archive_credentials_free_func_t_callback<
    F: AeronArchiveCredentialsFreeFuncCallback,
>(
    credentials: *mut aeron_archive_encoded_credentials_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_credentials_free_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_credentials_free_func(credentials.into())
}
#[allow(dead_code)]
#[doc = "Callback to return encoded credentials so they may be reused or freed."]
#[doc = ""]
#[doc = " @param credentials to reuse or free"]
unsafe extern "C" fn aeron_archive_credentials_free_func_t_callback_for_once_closure<
    F: FnMut(AeronArchiveEncodedCredentials) -> (),
>(
    credentials: *mut aeron_archive_encoded_credentials_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_archive_credentials_free_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(credentials.into())
}
#[doc = "Callback to allow execution of a delegating invoker to be run."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveDelegatingInvokerFuncCallback {
    fn handle_aeron_archive_delegating_invoker_func(&mut self) -> ();
}
pub struct AeronArchiveDelegatingInvokerFuncLogger;
impl AeronArchiveDelegatingInvokerFuncCallback for AeronArchiveDelegatingInvokerFuncLogger {
    fn handle_aeron_archive_delegating_invoker_func(&mut self) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_delegating_invoker_func),
            [""].join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronArchiveDelegatingInvokerFuncLogger {}
unsafe impl Sync for AeronArchiveDelegatingInvokerFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_delegating_invoker_func_handler(
    ) -> Option<&'static Handler<AeronArchiveDelegatingInvokerFuncLogger>> {
        None::<&Handler<AeronArchiveDelegatingInvokerFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to allow execution of a delegating invoker to be run."]
unsafe extern "C" fn aeron_archive_delegating_invoker_func_t_callback<
    F: AeronArchiveDelegatingInvokerFuncCallback,
>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_delegating_invoker_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_delegating_invoker_func()
}
#[allow(dead_code)]
#[doc = "Callback to allow execution of a delegating invoker to be run."]
unsafe extern "C" fn aeron_archive_delegating_invoker_func_t_callback_for_once_closure<
    F: FnMut() -> (),
>(
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_archive_delegating_invoker_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure()
}
#[doc = "Callback to return recording descriptors."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveRecordingDescriptorConsumerFuncCallback {
    fn handle_aeron_archive_recording_descriptor_consumer_func(
        &mut self,
        recording_descriptor: AeronArchiveRecordingDescriptor,
    ) -> ();
}
pub struct AeronArchiveRecordingDescriptorConsumerFuncLogger;
impl AeronArchiveRecordingDescriptorConsumerFuncCallback
    for AeronArchiveRecordingDescriptorConsumerFuncLogger
{
    fn handle_aeron_archive_recording_descriptor_consumer_func(
        &mut self,
        recording_descriptor: AeronArchiveRecordingDescriptor,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_recording_descriptor_consumer_func),
            [format!(
                "{} : {:?}",
                stringify!(recording_descriptor),
                recording_descriptor
            )]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronArchiveRecordingDescriptorConsumerFuncLogger {}
unsafe impl Sync for AeronArchiveRecordingDescriptorConsumerFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_recording_descriptor_consumer_func_handler(
    ) -> Option<&'static Handler<AeronArchiveRecordingDescriptorConsumerFuncLogger>> {
        None::<&Handler<AeronArchiveRecordingDescriptorConsumerFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return recording descriptors."]
unsafe extern "C" fn aeron_archive_recording_descriptor_consumer_func_t_callback<
    F: AeronArchiveRecordingDescriptorConsumerFuncCallback,
>(
    recording_descriptor: *mut aeron_archive_recording_descriptor_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_recording_descriptor_consumer_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_recording_descriptor_consumer_func(recording_descriptor.into())
}
#[allow(dead_code)]
#[doc = "Callback to return recording descriptors."]
unsafe extern "C" fn aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure<
    F: FnMut(AeronArchiveRecordingDescriptor) -> (),
>(
    recording_descriptor: *mut aeron_archive_recording_descriptor_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(
                aeron_archive_recording_descriptor_consumer_func_t_callback_for_once_closure
            )
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(recording_descriptor.into())
}
#[doc = "Callback to return recording subscription descriptors."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveRecordingSubscriptionDescriptorConsumerFuncCallback {
    fn handle_aeron_archive_recording_subscription_descriptor_consumer_func(
        &mut self,
        recording_subscription_descriptor: AeronArchiveRecordingSubscriptionDescriptor,
    ) -> ();
}
pub struct AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger;
impl AeronArchiveRecordingSubscriptionDescriptorConsumerFuncCallback
    for AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger
{
    fn handle_aeron_archive_recording_subscription_descriptor_consumer_func(
        &mut self,
        recording_subscription_descriptor: AeronArchiveRecordingSubscriptionDescriptor,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_recording_subscription_descriptor_consumer_func),
            [format!(
                "{} : {:?}",
                stringify!(recording_subscription_descriptor),
                recording_subscription_descriptor
            )]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger {}
unsafe impl Sync for AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_recording_subscription_descriptor_consumer_func_handler(
    ) -> Option<&'static Handler<AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger>>
    {
        None::<&Handler<AeronArchiveRecordingSubscriptionDescriptorConsumerFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return recording subscription descriptors."]
unsafe extern "C" fn aeron_archive_recording_subscription_descriptor_consumer_func_t_callback<
    F: AeronArchiveRecordingSubscriptionDescriptorConsumerFuncCallback,
>(
    recording_subscription_descriptor: *mut aeron_archive_recording_subscription_descriptor_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_recording_subscription_descriptor_consumer_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_recording_subscription_descriptor_consumer_func(
        recording_subscription_descriptor.into(),
    )
}
#[allow(dead_code)]
#[doc = "Callback to return recording subscription descriptors."]
unsafe extern "C" fn aeron_archive_recording_subscription_descriptor_consumer_func_t_callback_for_once_closure<
    F: FnMut(AeronArchiveRecordingSubscriptionDescriptor) -> (),
>(
    recording_subscription_descriptor: *mut aeron_archive_recording_subscription_descriptor_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log :: debug ! ("calling {}" , stringify ! (aeron_archive_recording_subscription_descriptor_consumer_func_t_callback_for_once_closure));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(recording_subscription_descriptor.into())
}
#[doc = "Callback to return recording signals."]
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronArchiveRecordingSignalConsumerFuncCallback {
    fn handle_aeron_archive_recording_signal_consumer_func(
        &mut self,
        recording_signal: AeronArchiveRecordingSignal,
    ) -> ();
}
pub struct AeronArchiveRecordingSignalConsumerFuncLogger;
impl AeronArchiveRecordingSignalConsumerFuncCallback
    for AeronArchiveRecordingSignalConsumerFuncLogger
{
    fn handle_aeron_archive_recording_signal_consumer_func(
        &mut self,
        recording_signal: AeronArchiveRecordingSignal,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_archive_recording_signal_consumer_func),
            [format!(
                "{} : {:?}",
                stringify!(recording_signal),
                recording_signal
            )]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronArchiveRecordingSignalConsumerFuncLogger {}
unsafe impl Sync for AeronArchiveRecordingSignalConsumerFuncLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_archive_recording_signal_consumer_func_handler(
    ) -> Option<&'static Handler<AeronArchiveRecordingSignalConsumerFuncLogger>> {
        None::<&Handler<AeronArchiveRecordingSignalConsumerFuncLogger>>
    }
}
#[allow(dead_code)]
#[doc = "Callback to return recording signals."]
unsafe extern "C" fn aeron_archive_recording_signal_consumer_func_t_callback<
    F: AeronArchiveRecordingSignalConsumerFuncCallback,
>(
    recording_signal: *mut aeron_archive_recording_signal_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(handle_aeron_archive_recording_signal_consumer_func)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_archive_recording_signal_consumer_func(recording_signal.into())
}
#[allow(dead_code)]
#[doc = "Callback to return recording signals."]
unsafe extern "C" fn aeron_archive_recording_signal_consumer_func_t_callback_for_once_closure<
    F: FnMut(AeronArchiveRecordingSignal) -> (),
>(
    recording_signal: *mut aeron_archive_recording_signal_t,
    clientd: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_archive_recording_signal_consumer_func_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(recording_signal.into())
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronUriParseCallbackCallback {
    fn handle_aeron_uri_parse_callback(&mut self, key: &str, value: &str) -> ::std::os::raw::c_int;
}
pub struct AeronUriParseCallbackLogger;
impl AeronUriParseCallbackCallback for AeronUriParseCallbackLogger {
    fn handle_aeron_uri_parse_callback(&mut self, key: &str, value: &str) -> ::std::os::raw::c_int {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_uri_parse_callback),
            [
                format!("{} : {:?}", stringify!(key), key),
                format!("{} : {:?}", stringify!(value), value)
            ]
            .join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronUriParseCallbackLogger {}
unsafe impl Sync for AeronUriParseCallbackLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_uri_parse_callback_handler() -> Option<&'static Handler<AeronUriParseCallbackLogger>>
    {
        None::<&Handler<AeronUriParseCallbackLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_uri_parse_callback_t_callback<F: AeronUriParseCallbackCallback>(
    clientd: *mut ::std::os::raw::c_void,
    key: *const ::std::os::raw::c_char,
    value: *const ::std::os::raw::c_char,
) -> ::std::os::raw::c_int {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_uri_parse_callback));
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_uri_parse_callback(
        if key.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(key).to_str().unwrap() }
        },
        if value.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(value).to_str().unwrap() }
        },
    )
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_uri_parse_callback_t_callback_for_once_closure<
    F: FnMut(&str, &str) -> ::std::os::raw::c_int,
>(
    clientd: *mut ::std::os::raw::c_void,
    key: *const ::std::os::raw::c_char,
    value: *const ::std::os::raw::c_char,
) -> ::std::os::raw::c_int {
    #[cfg(debug_assertions)]
    if clientd.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_uri_parse_callback_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(
        if key.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(key).to_str().unwrap() }
        },
        if value.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(value).to_str().unwrap() }
        },
    )
}
