
#[derive(Clone)]
pub struct AeronBroadcastDescriptor {
    inner: std::rc::Rc<ManagedCResource<aeron_broadcast_descriptor_t>>,
}
impl core::fmt::Debug for AeronBroadcastDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronBroadcastDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronBroadcastDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(tail_intent_counter), &self.tail_intent_counter())
                .field(stringify!(tail_counter), &self.tail_counter())
                .field(stringify!(latest_counter), &self.latest_counter())
                .finish()
        }
    }
}
impl AeronBroadcastDescriptor {
    #[inline]
    pub fn new(
        tail_intent_counter: i64,
        tail_counter: i64,
        latest_counter: i64,
        pad: [u8; 104usize],
    ) -> Result<Self, AeronCError> {
        let drop_copies_closure = std::rc::Rc::new(std::cell::RefCell::new(Some(|| {})));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_broadcast_descriptor_t {
                    tail_intent_counter: tail_intent_counter.into(),
                    tail_counter: tail_counter.into(),
                    latest_counter: latest_counter.into(),
                    pad: pad.into(),
                };
                let inner_ptr: *mut aeron_broadcast_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_broadcast_descriptor_t)
                );
                let inst: aeron_broadcast_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_broadcast_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn tail_intent_counter(&self) -> i64 {
        self.tail_intent_counter.into()
    }
    #[inline]
    pub fn tail_counter(&self) -> i64 {
        self.tail_counter.into()
    }
    #[inline]
    pub fn latest_counter(&self) -> i64 {
        self.latest_counter.into()
    }
    #[inline]
    pub fn pad(&self) -> [u8; 104usize] {
        self.pad.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_broadcast_descriptor_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronBroadcastDescriptor {
    type Target = aeron_broadcast_descriptor_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_broadcast_descriptor_t> for AeronBroadcastDescriptor {
    #[inline]
    fn from(value: *mut aeron_broadcast_descriptor_t) -> Self {
        AeronBroadcastDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronBroadcastDescriptor> for *mut aeron_broadcast_descriptor_t {
    #[inline]
    fn from(value: AeronBroadcastDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronBroadcastDescriptor> for *mut aeron_broadcast_descriptor_t {
    #[inline]
    fn from(value: &AeronBroadcastDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronBroadcastDescriptor> for aeron_broadcast_descriptor_t {
    #[inline]
    fn from(value: AeronBroadcastDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_broadcast_descriptor_t> for AeronBroadcastDescriptor {
    #[inline]
    fn from(value: *const aeron_broadcast_descriptor_t) -> Self {
        AeronBroadcastDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_broadcast_descriptor_t> for AeronBroadcastDescriptor {
    #[inline]
    fn from(mut value: aeron_broadcast_descriptor_t) -> Self {
        AeronBroadcastDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_broadcast_descriptor_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronBroadcastDescriptor {
    fn default() -> Self {
        AeronBroadcastDescriptor::new_zeroed().expect("failed to create struct")
    }
}
impl AeronBroadcastDescriptor {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
use crate::AeronErrorType::Unknown;
use std::any::Any;
#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
#[doc = " A custom struct for managing C resources with automatic cleanup."]
#[doc = ""]
#[doc = " It handles initialisation and clean-up of the resource and ensures that resources"]
#[doc = " are properly released when they go out of scope."]
#[allow(dead_code)]
pub struct ManagedCResource<T> {
    resource: *mut T,
    cleanup: Option<Box<dyn FnMut(*mut *mut T) -> i32>>,
    cleanup_struct: bool,
    borrowed: bool,
    #[doc = " if someone externally rusteron calls close"]
    close_already_called: std::cell::Cell<bool>,
    #[doc = " if there is a c method to verify it someone has closed it, only few structs have this functionality"]
    check_for_is_closed: Option<Box<dyn Fn(*mut T) -> bool>>,
    #[doc = " this will be called if closed hasn't already happened even if its borrowed"]
    auto_close: std::cell::Cell<bool>,
    dependencies: UnsafeCell<Vec<std::rc::Rc<dyn Any>>>,
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
        check_for_is_closed: Option<Box<dyn Fn(*mut T) -> bool>>,
    ) -> Result<Self, AeronCError> {
        let mut resource: *mut T = std::ptr::null_mut();
        let result = init(&mut resource);
        if result < 0 || resource.is_null() {
            return Err(AeronCError::from_code(result));
        }
        let result = Self {
            resource,
            cleanup,
            cleanup_struct,
            borrowed: false,
            close_already_called: std::cell::Cell::new(false),
            check_for_is_closed,
            auto_close: std::cell::Cell::new(false),
            dependencies: UnsafeCell::new(vec![]),
        };
        #[cfg(feature = "extra-logging")]
        log::debug!("created c resource: {:?}", result);
        Ok(result)
    }
    pub fn is_closed_already_called(&self) -> bool {
        self.close_already_called.get()
            || self.resource.is_null()
            || self
                .check_for_is_closed
                .as_ref()
                .map_or(false, |f| f(self.resource))
    }
    pub fn new_borrowed(
        value: *const T,
        check_for_is_closed: Option<Box<dyn Fn(*mut T) -> bool>>,
    ) -> Self {
        Self {
            resource: value as *mut _,
            cleanup: None,
            cleanup_struct: false,
            borrowed: true,
            close_already_called: std::cell::Cell::new(false),
            check_for_is_closed,
            auto_close: std::cell::Cell::new(false),
            dependencies: UnsafeCell::new(vec![]),
        }
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
    pub fn add_dependency<D: Any>(&self, dep: D) {
        unsafe {
            (*self.dependencies.get()).push(std::rc::Rc::new(dep));
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
            if !self.borrowed {
                self.close_already_called.set(true);
                let resource = if already_closed {
                    self.resource
                } else {
                    self.resource.clone()
                };
                if !already_closed {
                    #[cfg(feature = "extra-logging")]
                    log::debug!("closing c resource: {:?}", self);
                    let _ = self.close();
                }
                if self.cleanup_struct {
                    #[cfg(feature = "extra-logging")]
                    log::debug!("closing rust struct resource: {:?}", resource);
                    unsafe {
                        let _ = Box::from_raw(resource);
                    }
                }
            }
        }
    }
}
#[derive(Debug, PartialOrd, Eq, PartialEq, Clone)]
pub enum AeronErrorType {
    NullOrNotConnected,
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
            AeronErrorType::NullOrNotConnected => -1,
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
            -1 => AeronErrorType::NullOrNotConnected,
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
            AeronErrorType::NullOrNotConnected => "Null Value or Not Connected",
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
#[doc = " Use `get_message()` to retrieve a human-readable message, if available."]
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
impl std::fmt::Display for AeronCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Aeron error {}: {:?}", self.code, self.kind())
    }
}
impl std::fmt::Debug for AeronCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AeronCError")
            .field("code", &self.code)
            .field("kind", &self.kind())
            .finish()
    }
}
impl std::error::Error for AeronCError {}
#[doc = " # Handler"]
#[doc = ""]
#[doc = " `Handler` is a struct that wraps a raw pointer and a drop flag."]
#[doc = ""]
#[doc = " **Important:** `Handler` does not get dropped automatically."]
#[doc = " You need to call the `release` method if you want to clear the memory manually."]
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
                log::debug!("dropping handler {:?}", self.raw_ptr);
                let _ = Box::from_raw(self.raw_ptr as *mut Box<T>);
                self.should_drop = false;
            }
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
#[derive(Clone)]
pub struct AeronBroadcastReceiver {
    inner: std::rc::Rc<ManagedCResource<aeron_broadcast_receiver_t>>,
}
impl core::fmt::Debug for AeronBroadcastReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronBroadcastReceiver))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronBroadcastReceiver))
                .field("inner", &self.inner)
                .field(stringify!(capacity), &self.capacity())
                .field(stringify!(mask), &self.mask())
                .field(stringify!(record_offset), &self.record_offset())
                .field(stringify!(cursor), &self.cursor())
                .field(stringify!(next_record), &self.next_record())
                .finish()
        }
    }
}
impl AeronBroadcastReceiver {
    #[inline]
    pub fn new(
        scratch_buffer: [u8; 4096usize],
        buffer: *mut u8,
        descriptor: &AeronBroadcastDescriptor,
        capacity: usize,
        mask: usize,
        record_offset: usize,
        cursor: i64,
        next_record: i64,
        lapped_count: ::std::os::raw::c_long,
    ) -> Result<Self, AeronCError> {
        let descriptor_copy = descriptor.clone();
        let drop_copies_closure =
            std::rc::Rc::new(std::cell::RefCell::new(Some(|| drop(descriptor_copy))));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_broadcast_receiver_t {
                    scratch_buffer: scratch_buffer.into(),
                    buffer: buffer.into(),
                    descriptor: descriptor.into(),
                    capacity: capacity.into(),
                    mask: mask.into(),
                    record_offset: record_offset.into(),
                    cursor: cursor.into(),
                    next_record: next_record.into(),
                    lapped_count: lapped_count.into(),
                };
                let inner_ptr: *mut aeron_broadcast_receiver_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_broadcast_receiver_t)
                );
                let inst: aeron_broadcast_receiver_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_broadcast_receiver_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn scratch_buffer(&self) -> [u8; 4096usize] {
        self.scratch_buffer.into()
    }
    #[inline]
    pub fn buffer(&self) -> *mut u8 {
        self.buffer.into()
    }
    #[inline]
    pub fn descriptor(&self) -> AeronBroadcastDescriptor {
        self.descriptor.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn mask(&self) -> usize {
        self.mask.into()
    }
    #[inline]
    pub fn record_offset(&self) -> usize {
        self.record_offset.into()
    }
    #[inline]
    pub fn cursor(&self) -> i64 {
        self.cursor.into()
    }
    #[inline]
    pub fn next_record(&self) -> i64 {
        self.next_record.into()
    }
    #[inline]
    pub fn lapped_count(&self) -> ::std::os::raw::c_long {
        self.lapped_count.into()
    }
    #[inline]
    pub fn init(
        &self,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_broadcast_receiver_init(self.get_inner(), buffer.into(), length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn receive<
        AeronBroadcastReceiverHandlerHandlerImpl: AeronBroadcastReceiverHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronBroadcastReceiverHandlerHandlerImpl>>,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_broadcast_receiver_receive(
                self.get_inner(),
                {
                    let callback: aeron_broadcast_receiver_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_broadcast_receiver_handler_t_callback::<
                                AeronBroadcastReceiverHandlerHandlerImpl,
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
    pub fn receive_once<AeronBroadcastReceiverHandlerHandlerImpl: FnMut(i32, &mut [u8]) -> ()>(
        &self,
        mut handler: AeronBroadcastReceiverHandlerHandlerImpl,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_broadcast_receiver_receive(
                self.get_inner(),
                Some(
                    aeron_broadcast_receiver_handler_t_callback_for_once_closure::<
                        AeronBroadcastReceiverHandlerHandlerImpl,
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
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_broadcast_receiver_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronBroadcastReceiver {
    type Target = aeron_broadcast_receiver_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_broadcast_receiver_t> for AeronBroadcastReceiver {
    #[inline]
    fn from(value: *mut aeron_broadcast_receiver_t) -> Self {
        AeronBroadcastReceiver {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronBroadcastReceiver> for *mut aeron_broadcast_receiver_t {
    #[inline]
    fn from(value: AeronBroadcastReceiver) -> Self {
        value.get_inner()
    }
}
impl From<&AeronBroadcastReceiver> for *mut aeron_broadcast_receiver_t {
    #[inline]
    fn from(value: &AeronBroadcastReceiver) -> Self {
        value.get_inner()
    }
}
impl From<AeronBroadcastReceiver> for aeron_broadcast_receiver_t {
    #[inline]
    fn from(value: AeronBroadcastReceiver) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_broadcast_receiver_t> for AeronBroadcastReceiver {
    #[inline]
    fn from(value: *const aeron_broadcast_receiver_t) -> Self {
        AeronBroadcastReceiver {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_broadcast_receiver_t> for AeronBroadcastReceiver {
    #[inline]
    fn from(mut value: aeron_broadcast_receiver_t) -> Self {
        AeronBroadcastReceiver {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_broadcast_receiver_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronBroadcastReceiver {
    fn default() -> Self {
        AeronBroadcastReceiver::new_zeroed().expect("failed to create struct")
    }
}
impl AeronBroadcastReceiver {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronBroadcastRecordDescriptor {
    inner: std::rc::Rc<ManagedCResource<aeron_broadcast_record_descriptor_t>>,
}
impl core::fmt::Debug for AeronBroadcastRecordDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronBroadcastRecordDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronBroadcastRecordDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .field(stringify!(msg_type_id), &self.msg_type_id())
                .finish()
        }
    }
}
impl AeronBroadcastRecordDescriptor {
    #[inline]
    pub fn new(length: i32, msg_type_id: i32) -> Result<Self, AeronCError> {
        let drop_copies_closure = std::rc::Rc::new(std::cell::RefCell::new(Some(|| {})));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_broadcast_record_descriptor_t {
                    length: length.into(),
                    msg_type_id: msg_type_id.into(),
                };
                let inner_ptr: *mut aeron_broadcast_record_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_broadcast_record_descriptor_t)
                );
                let inst: aeron_broadcast_record_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_broadcast_record_descriptor_t =
                    Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn length(&self) -> i32 {
        self.length.into()
    }
    #[inline]
    pub fn msg_type_id(&self) -> i32 {
        self.msg_type_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_broadcast_record_descriptor_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronBroadcastRecordDescriptor {
    type Target = aeron_broadcast_record_descriptor_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_broadcast_record_descriptor_t> for AeronBroadcastRecordDescriptor {
    #[inline]
    fn from(value: *mut aeron_broadcast_record_descriptor_t) -> Self {
        AeronBroadcastRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronBroadcastRecordDescriptor> for *mut aeron_broadcast_record_descriptor_t {
    #[inline]
    fn from(value: AeronBroadcastRecordDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronBroadcastRecordDescriptor> for *mut aeron_broadcast_record_descriptor_t {
    #[inline]
    fn from(value: &AeronBroadcastRecordDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronBroadcastRecordDescriptor> for aeron_broadcast_record_descriptor_t {
    #[inline]
    fn from(value: AeronBroadcastRecordDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_broadcast_record_descriptor_t> for AeronBroadcastRecordDescriptor {
    #[inline]
    fn from(value: *const aeron_broadcast_record_descriptor_t) -> Self {
        AeronBroadcastRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_broadcast_record_descriptor_t> for AeronBroadcastRecordDescriptor {
    #[inline]
    fn from(mut value: aeron_broadcast_record_descriptor_t) -> Self {
        AeronBroadcastRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_broadcast_record_descriptor_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronBroadcastRecordDescriptor {
    fn default() -> Self {
        AeronBroadcastRecordDescriptor::new_zeroed().expect("failed to create struct")
    }
}
impl AeronBroadcastRecordDescriptor {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronBroadcastTransmitter {
    inner: std::rc::Rc<ManagedCResource<aeron_broadcast_transmitter_t>>,
}
impl core::fmt::Debug for AeronBroadcastTransmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronBroadcastTransmitter))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronBroadcastTransmitter))
                .field("inner", &self.inner)
                .field(stringify!(capacity), &self.capacity())
                .field(stringify!(max_message_length), &self.max_message_length())
                .finish()
        }
    }
}
impl AeronBroadcastTransmitter {
    #[inline]
    pub fn new(
        buffer: *mut u8,
        descriptor: &AeronBroadcastDescriptor,
        capacity: usize,
        max_message_length: usize,
    ) -> Result<Self, AeronCError> {
        let descriptor_copy = descriptor.clone();
        let drop_copies_closure =
            std::rc::Rc::new(std::cell::RefCell::new(Some(|| drop(descriptor_copy))));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_broadcast_transmitter_t {
                    buffer: buffer.into(),
                    descriptor: descriptor.into(),
                    capacity: capacity.into(),
                    max_message_length: max_message_length.into(),
                };
                let inner_ptr: *mut aeron_broadcast_transmitter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_broadcast_transmitter_t)
                );
                let inst: aeron_broadcast_transmitter_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_broadcast_transmitter_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn buffer(&self) -> *mut u8 {
        self.buffer.into()
    }
    #[inline]
    pub fn descriptor(&self) -> AeronBroadcastDescriptor {
        self.descriptor.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn max_message_length(&self) -> usize {
        self.max_message_length.into()
    }
    #[inline]
    pub fn init(
        &self,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result =
                aeron_broadcast_transmitter_init(self.get_inner(), buffer.into(), length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn transmit(
        &self,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_broadcast_transmitter_transmit(
                self.get_inner(),
                msg_type_id.into(),
                msg.into(),
                length.into(),
            );
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_broadcast_transmitter_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronBroadcastTransmitter {
    type Target = aeron_broadcast_transmitter_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_broadcast_transmitter_t> for AeronBroadcastTransmitter {
    #[inline]
    fn from(value: *mut aeron_broadcast_transmitter_t) -> Self {
        AeronBroadcastTransmitter {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronBroadcastTransmitter> for *mut aeron_broadcast_transmitter_t {
    #[inline]
    fn from(value: AeronBroadcastTransmitter) -> Self {
        value.get_inner()
    }
}
impl From<&AeronBroadcastTransmitter> for *mut aeron_broadcast_transmitter_t {
    #[inline]
    fn from(value: &AeronBroadcastTransmitter) -> Self {
        value.get_inner()
    }
}
impl From<AeronBroadcastTransmitter> for aeron_broadcast_transmitter_t {
    #[inline]
    fn from(value: AeronBroadcastTransmitter) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_broadcast_transmitter_t> for AeronBroadcastTransmitter {
    #[inline]
    fn from(value: *const aeron_broadcast_transmitter_t) -> Self {
        AeronBroadcastTransmitter {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_broadcast_transmitter_t> for AeronBroadcastTransmitter {
    #[inline]
    fn from(mut value: aeron_broadcast_transmitter_t) -> Self {
        AeronBroadcastTransmitter {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_broadcast_transmitter_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronBroadcastTransmitter {
    fn default() -> Self {
        AeronBroadcastTransmitter::new_zeroed().expect("failed to create struct")
    }
}
impl AeronBroadcastTransmitter {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronMpscRb {
    inner: std::rc::Rc<ManagedCResource<aeron_mpsc_rb_t>>,
}
impl core::fmt::Debug for AeronMpscRb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronMpscRb))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronMpscRb))
                .field("inner", &self.inner)
                .field(stringify!(capacity), &self.capacity())
                .field(stringify!(max_message_length), &self.max_message_length())
                .finish()
        }
    }
}
impl AeronMpscRb {
    #[inline]
    pub fn new(
        buffer: *mut u8,
        descriptor: &AeronRbDescriptor,
        capacity: usize,
        max_message_length: usize,
    ) -> Result<Self, AeronCError> {
        let descriptor_copy = descriptor.clone();
        let drop_copies_closure =
            std::rc::Rc::new(std::cell::RefCell::new(Some(|| drop(descriptor_copy))));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_mpsc_rb_t {
                    buffer: buffer.into(),
                    descriptor: descriptor.into(),
                    capacity: capacity.into(),
                    max_message_length: max_message_length.into(),
                };
                let inner_ptr: *mut aeron_mpsc_rb_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_mpsc_rb_t)
                );
                let inst: aeron_mpsc_rb_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_mpsc_rb_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn buffer(&self) -> *mut u8 {
        self.buffer.into()
    }
    #[inline]
    pub fn descriptor(&self) -> AeronRbDescriptor {
        self.descriptor.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn max_message_length(&self) -> usize {
        self.max_message_length.into()
    }
    #[inline]
    pub fn init(
        &self,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_mpsc_rb_init(self.get_inner(), buffer.into(), length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn write(
        &self,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> aeron_rb_write_result_t {
        unsafe {
            let result = aeron_mpsc_rb_write(
                self.get_inner(),
                msg_type_id.into(),
                msg.into(),
                length.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn try_claim(&self, msg_type_id: i32, length: usize) -> i32 {
        unsafe {
            let result =
                aeron_mpsc_rb_try_claim(self.get_inner(), msg_type_id.into(), length.into());
            result.into()
        }
    }
    #[inline]
    pub fn commit(&self, offset: i32) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_mpsc_rb_commit(self.get_inner(), offset.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn abort(&self, offset: i32) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_mpsc_rb_abort(self.get_inner(), offset.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn read<AeronRbHandlerHandlerImpl: AeronRbHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronRbHandlerHandlerImpl>>,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_mpsc_rb_read(
                self.get_inner(),
                {
                    let callback: aeron_rb_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_rb_handler_t_callback::<AeronRbHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                message_count_limit.into(),
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
        AeronRbHandlerHandlerImpl: FnMut(i32, *const ::std::os::raw::c_void, usize) -> (),
    >(
        &self,
        mut handler: AeronRbHandlerHandlerImpl,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_mpsc_rb_read(
                self.get_inner(),
                Some(aeron_rb_handler_t_callback_for_once_closure::<AeronRbHandlerHandlerImpl>),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn controlled_read<
        AeronRbControlledHandlerHandlerImpl: AeronRbControlledHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronRbControlledHandlerHandlerImpl>>,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_mpsc_rb_controlled_read(
                self.get_inner(),
                {
                    let callback: aeron_rb_controlled_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_rb_controlled_handler_t_callback::<
                                AeronRbControlledHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn controlled_read_once<
        AeronRbControlledHandlerHandlerImpl: FnMut(i32, *const ::std::os::raw::c_void, usize) -> aeron_rb_read_action_t,
    >(
        &self,
        mut handler: AeronRbControlledHandlerHandlerImpl,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_mpsc_rb_controlled_read(
                self.get_inner(),
                Some(
                    aeron_rb_controlled_handler_t_callback_for_once_closure::<
                        AeronRbControlledHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn next_correlation_id(&self) -> i64 {
        unsafe {
            let result = aeron_mpsc_rb_next_correlation_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn consumer_heartbeat_time(&self, now_ms: i64) -> () {
        unsafe {
            let result = aeron_mpsc_rb_consumer_heartbeat_time(self.get_inner(), now_ms.into());
            result.into()
        }
    }
    #[inline]
    pub fn consumer_heartbeat_time_value(&self) -> i64 {
        unsafe {
            let result = aeron_mpsc_rb_consumer_heartbeat_time_value(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn unblock(&self) -> bool {
        unsafe {
            let result = aeron_mpsc_rb_unblock(self.get_inner());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_mpsc_rb_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronMpscRb {
    type Target = aeron_mpsc_rb_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_mpsc_rb_t> for AeronMpscRb {
    #[inline]
    fn from(value: *mut aeron_mpsc_rb_t) -> Self {
        AeronMpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronMpscRb> for *mut aeron_mpsc_rb_t {
    #[inline]
    fn from(value: AeronMpscRb) -> Self {
        value.get_inner()
    }
}
impl From<&AeronMpscRb> for *mut aeron_mpsc_rb_t {
    #[inline]
    fn from(value: &AeronMpscRb) -> Self {
        value.get_inner()
    }
}
impl From<AeronMpscRb> for aeron_mpsc_rb_t {
    #[inline]
    fn from(value: AeronMpscRb) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_mpsc_rb_t> for AeronMpscRb {
    #[inline]
    fn from(value: *const aeron_mpsc_rb_t) -> Self {
        AeronMpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_mpsc_rb_t> for AeronMpscRb {
    #[inline]
    fn from(mut value: aeron_mpsc_rb_t) -> Self {
        AeronMpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_mpsc_rb_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronMpscRb {
    fn default() -> Self {
        AeronMpscRb::new_zeroed().expect("failed to create struct")
    }
}
impl AeronMpscRb {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronRbDescriptor {
    inner: std::rc::Rc<ManagedCResource<aeron_rb_descriptor_t>>,
}
impl core::fmt::Debug for AeronRbDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronRbDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronRbDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(tail_position), &self.tail_position())
                .field(stringify!(head_cache_position), &self.head_cache_position())
                .field(stringify!(head_position), &self.head_position())
                .field(stringify!(correlation_counter), &self.correlation_counter())
                .field(stringify!(consumer_heartbeat), &self.consumer_heartbeat())
                .finish()
        }
    }
}
impl AeronRbDescriptor {
    #[inline]
    pub fn new(
        begin_pad: [u8; 128usize],
        tail_position: i64,
        tail_pad: [u8; 120usize],
        head_cache_position: i64,
        head_cache_pad: [u8; 120usize],
        head_position: i64,
        head_pad: [u8; 120usize],
        correlation_counter: i64,
        correlation_counter_pad: [u8; 120usize],
        consumer_heartbeat: i64,
        consumer_heartbeat_pad: [u8; 120usize],
    ) -> Result<Self, AeronCError> {
        let drop_copies_closure = std::rc::Rc::new(std::cell::RefCell::new(Some(|| {})));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_rb_descriptor_t {
                    begin_pad: begin_pad.into(),
                    tail_position: tail_position.into(),
                    tail_pad: tail_pad.into(),
                    head_cache_position: head_cache_position.into(),
                    head_cache_pad: head_cache_pad.into(),
                    head_position: head_position.into(),
                    head_pad: head_pad.into(),
                    correlation_counter: correlation_counter.into(),
                    correlation_counter_pad: correlation_counter_pad.into(),
                    consumer_heartbeat: consumer_heartbeat.into(),
                    consumer_heartbeat_pad: consumer_heartbeat_pad.into(),
                };
                let inner_ptr: *mut aeron_rb_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_rb_descriptor_t)
                );
                let inst: aeron_rb_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_rb_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn begin_pad(&self) -> [u8; 128usize] {
        self.begin_pad.into()
    }
    #[inline]
    pub fn tail_position(&self) -> i64 {
        self.tail_position.into()
    }
    #[inline]
    pub fn tail_pad(&self) -> [u8; 120usize] {
        self.tail_pad.into()
    }
    #[inline]
    pub fn head_cache_position(&self) -> i64 {
        self.head_cache_position.into()
    }
    #[inline]
    pub fn head_cache_pad(&self) -> [u8; 120usize] {
        self.head_cache_pad.into()
    }
    #[inline]
    pub fn head_position(&self) -> i64 {
        self.head_position.into()
    }
    #[inline]
    pub fn head_pad(&self) -> [u8; 120usize] {
        self.head_pad.into()
    }
    #[inline]
    pub fn correlation_counter(&self) -> i64 {
        self.correlation_counter.into()
    }
    #[inline]
    pub fn correlation_counter_pad(&self) -> [u8; 120usize] {
        self.correlation_counter_pad.into()
    }
    #[inline]
    pub fn consumer_heartbeat(&self) -> i64 {
        self.consumer_heartbeat.into()
    }
    #[inline]
    pub fn consumer_heartbeat_pad(&self) -> [u8; 120usize] {
        self.consumer_heartbeat_pad.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_rb_descriptor_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronRbDescriptor {
    type Target = aeron_rb_descriptor_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_rb_descriptor_t> for AeronRbDescriptor {
    #[inline]
    fn from(value: *mut aeron_rb_descriptor_t) -> Self {
        AeronRbDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronRbDescriptor> for *mut aeron_rb_descriptor_t {
    #[inline]
    fn from(value: AeronRbDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronRbDescriptor> for *mut aeron_rb_descriptor_t {
    #[inline]
    fn from(value: &AeronRbDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronRbDescriptor> for aeron_rb_descriptor_t {
    #[inline]
    fn from(value: AeronRbDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_rb_descriptor_t> for AeronRbDescriptor {
    #[inline]
    fn from(value: *const aeron_rb_descriptor_t) -> Self {
        AeronRbDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_rb_descriptor_t> for AeronRbDescriptor {
    #[inline]
    fn from(mut value: aeron_rb_descriptor_t) -> Self {
        AeronRbDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_rb_descriptor_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronRbDescriptor {
    fn default() -> Self {
        AeronRbDescriptor::new_zeroed().expect("failed to create struct")
    }
}
impl AeronRbDescriptor {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronRbRecordDescriptor {
    inner: std::rc::Rc<ManagedCResource<aeron_rb_record_descriptor_t>>,
}
impl core::fmt::Debug for AeronRbRecordDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronRbRecordDescriptor))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronRbRecordDescriptor))
                .field("inner", &self.inner)
                .field(stringify!(length), &self.length())
                .field(stringify!(msg_type_id), &self.msg_type_id())
                .finish()
        }
    }
}
impl AeronRbRecordDescriptor {
    #[inline]
    pub fn new(length: i32, msg_type_id: i32) -> Result<Self, AeronCError> {
        let drop_copies_closure = std::rc::Rc::new(std::cell::RefCell::new(Some(|| {})));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_rb_record_descriptor_t {
                    length: length.into(),
                    msg_type_id: msg_type_id.into(),
                };
                let inner_ptr: *mut aeron_rb_record_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_rb_record_descriptor_t)
                );
                let inst: aeron_rb_record_descriptor_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_rb_record_descriptor_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn length(&self) -> i32 {
        self.length.into()
    }
    #[inline]
    pub fn msg_type_id(&self) -> i32 {
        self.msg_type_id.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_rb_record_descriptor_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronRbRecordDescriptor {
    type Target = aeron_rb_record_descriptor_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_rb_record_descriptor_t> for AeronRbRecordDescriptor {
    #[inline]
    fn from(value: *mut aeron_rb_record_descriptor_t) -> Self {
        AeronRbRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronRbRecordDescriptor> for *mut aeron_rb_record_descriptor_t {
    #[inline]
    fn from(value: AeronRbRecordDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<&AeronRbRecordDescriptor> for *mut aeron_rb_record_descriptor_t {
    #[inline]
    fn from(value: &AeronRbRecordDescriptor) -> Self {
        value.get_inner()
    }
}
impl From<AeronRbRecordDescriptor> for aeron_rb_record_descriptor_t {
    #[inline]
    fn from(value: AeronRbRecordDescriptor) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_rb_record_descriptor_t> for AeronRbRecordDescriptor {
    #[inline]
    fn from(value: *const aeron_rb_record_descriptor_t) -> Self {
        AeronRbRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_rb_record_descriptor_t> for AeronRbRecordDescriptor {
    #[inline]
    fn from(mut value: aeron_rb_record_descriptor_t) -> Self {
        AeronRbRecordDescriptor {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_rb_record_descriptor_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronRbRecordDescriptor {
    fn default() -> Self {
        AeronRbRecordDescriptor::new_zeroed().expect("failed to create struct")
    }
}
impl AeronRbRecordDescriptor {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct AeronSpscRb {
    inner: std::rc::Rc<ManagedCResource<aeron_spsc_rb_t>>,
}
impl core::fmt::Debug for AeronSpscRb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(AeronSpscRb))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(AeronSpscRb))
                .field("inner", &self.inner)
                .field(stringify!(capacity), &self.capacity())
                .field(stringify!(max_message_length), &self.max_message_length())
                .finish()
        }
    }
}
impl AeronSpscRb {
    #[inline]
    pub fn new(
        buffer: *mut u8,
        descriptor: &AeronRbDescriptor,
        capacity: usize,
        max_message_length: usize,
    ) -> Result<Self, AeronCError> {
        let descriptor_copy = descriptor.clone();
        let drop_copies_closure =
            std::rc::Rc::new(std::cell::RefCell::new(Some(|| drop(descriptor_copy))));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = aeron_spsc_rb_t {
                    buffer: buffer.into(),
                    descriptor: descriptor.into(),
                    capacity: capacity.into(),
                    max_message_length: max_message_length.into(),
                };
                let inner_ptr: *mut aeron_spsc_rb_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(aeron_spsc_rb_t)
                );
                let inst: aeron_spsc_rb_t = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut aeron_spsc_rb_t = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn buffer(&self) -> *mut u8 {
        self.buffer.into()
    }
    #[inline]
    pub fn descriptor(&self) -> AeronRbDescriptor {
        self.descriptor.into()
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity.into()
    }
    #[inline]
    pub fn max_message_length(&self) -> usize {
        self.max_message_length.into()
    }
    #[inline]
    pub fn init(
        &self,
        buffer: *mut ::std::os::raw::c_void,
        length: usize,
    ) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_spsc_rb_init(self.get_inner(), buffer.into(), length.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn write(
        &self,
        msg_type_id: i32,
        msg: *const ::std::os::raw::c_void,
        length: usize,
    ) -> aeron_rb_write_result_t {
        unsafe {
            let result = aeron_spsc_rb_write(
                self.get_inner(),
                msg_type_id.into(),
                msg.into(),
                length.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn writev(
        &self,
        msg_type_id: i32,
        iov: *const iovec,
        iovcnt: ::std::os::raw::c_int,
    ) -> aeron_rb_write_result_t {
        unsafe {
            let result = aeron_spsc_rb_writev(
                self.get_inner(),
                msg_type_id.into(),
                iov.into(),
                iovcnt.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn try_claim(&self, msg_type_id: i32, length: usize) -> i32 {
        unsafe {
            let result =
                aeron_spsc_rb_try_claim(self.get_inner(), msg_type_id.into(), length.into());
            result.into()
        }
    }
    #[inline]
    pub fn commit(&self, offset: i32) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_spsc_rb_commit(self.get_inner(), offset.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn abort(&self, offset: i32) -> Result<i32, AeronCError> {
        unsafe {
            let result = aeron_spsc_rb_abort(self.get_inner(), offset.into());
            if result < 0 {
                return Err(AeronCError::from_code(result));
            } else {
                return Ok(result);
            }
        }
    }
    #[inline]
    pub fn read<AeronRbHandlerHandlerImpl: AeronRbHandlerCallback>(
        &self,
        handler: Option<&Handler<AeronRbHandlerHandlerImpl>>,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_spsc_rb_read(
                self.get_inner(),
                {
                    let callback: aeron_rb_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(aeron_rb_handler_t_callback::<AeronRbHandlerHandlerImpl>)
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                message_count_limit.into(),
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
        AeronRbHandlerHandlerImpl: FnMut(i32, *const ::std::os::raw::c_void, usize) -> (),
    >(
        &self,
        mut handler: AeronRbHandlerHandlerImpl,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_spsc_rb_read(
                self.get_inner(),
                Some(aeron_rb_handler_t_callback_for_once_closure::<AeronRbHandlerHandlerImpl>),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn controlled_read<
        AeronRbControlledHandlerHandlerImpl: AeronRbControlledHandlerCallback,
    >(
        &self,
        handler: Option<&Handler<AeronRbControlledHandlerHandlerImpl>>,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_spsc_rb_controlled_read(
                self.get_inner(),
                {
                    let callback: aeron_rb_controlled_handler_t = if handler.is_none() {
                        None
                    } else {
                        Some(
                            aeron_rb_controlled_handler_t_callback::<
                                AeronRbControlledHandlerHandlerImpl,
                            >,
                        )
                    };
                    callback
                },
                handler
                    .map(|m| m.as_raw())
                    .unwrap_or_else(|| std::ptr::null_mut()),
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    #[doc = r""]
    #[doc = r""]
    #[doc = r" _NOTE: aeron must not store this closure and instead use it immediately. If not you will get undefined behaviour,"]
    #[doc = r"  use with care_"]
    pub fn controlled_read_once<
        AeronRbControlledHandlerHandlerImpl: FnMut(i32, *const ::std::os::raw::c_void, usize) -> aeron_rb_read_action_t,
    >(
        &self,
        mut handler: AeronRbControlledHandlerHandlerImpl,
        message_count_limit: usize,
    ) -> usize {
        unsafe {
            let result = aeron_spsc_rb_controlled_read(
                self.get_inner(),
                Some(
                    aeron_rb_controlled_handler_t_callback_for_once_closure::<
                        AeronRbControlledHandlerHandlerImpl,
                    >,
                ),
                &mut handler as *mut _ as *mut std::os::raw::c_void,
                message_count_limit.into(),
            );
            result.into()
        }
    }
    #[inline]
    pub fn next_correlation_id(&self) -> i64 {
        unsafe {
            let result = aeron_spsc_rb_next_correlation_id(self.get_inner());
            result.into()
        }
    }
    #[inline]
    pub fn consumer_heartbeat_time(&self, time_ms: i64) -> () {
        unsafe {
            let result = aeron_spsc_rb_consumer_heartbeat_time(self.get_inner(), time_ms.into());
            result.into()
        }
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut aeron_spsc_rb_t {
        self.inner.get()
    }
}
impl std::ops::Deref for AeronSpscRb {
    type Target = aeron_spsc_rb_t;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut aeron_spsc_rb_t> for AeronSpscRb {
    #[inline]
    fn from(value: *mut aeron_spsc_rb_t) -> Self {
        AeronSpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<AeronSpscRb> for *mut aeron_spsc_rb_t {
    #[inline]
    fn from(value: AeronSpscRb) -> Self {
        value.get_inner()
    }
}
impl From<&AeronSpscRb> for *mut aeron_spsc_rb_t {
    #[inline]
    fn from(value: &AeronSpscRb) -> Self {
        value.get_inner()
    }
}
impl From<AeronSpscRb> for aeron_spsc_rb_t {
    #[inline]
    fn from(value: AeronSpscRb) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const aeron_spsc_rb_t> for AeronSpscRb {
    #[inline]
    fn from(value: *const aeron_spsc_rb_t) -> Self {
        AeronSpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<aeron_spsc_rb_t> for AeronSpscRb {
    #[inline]
    fn from(mut value: aeron_spsc_rb_t) -> Self {
        AeronSpscRb {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut aeron_spsc_rb_t,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for AeronSpscRb {
    fn default() -> Self {
        AeronSpscRb::new_zeroed().expect("failed to create struct")
    }
}
impl AeronSpscRb {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[derive(Clone)]
pub struct Iovec {
    inner: std::rc::Rc<ManagedCResource<iovec>>,
}
impl core::fmt::Debug for Iovec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner.resource.is_null() {
            f.debug_struct(stringify!(Iovec))
                .field("inner", &"null")
                .finish()
        } else {
            f.debug_struct(stringify!(Iovec))
                .field("inner", &self.inner)
                .field(stringify!(iov_len), &self.iov_len())
                .finish()
        }
    }
}
impl Iovec {
    #[inline]
    pub fn new(iov_base: *mut ::std::os::raw::c_void, iov_len: usize) -> Result<Self, AeronCError> {
        let drop_copies_closure = std::rc::Rc::new(std::cell::RefCell::new(Some(|| {})));
        let r_constructor = ManagedCResource::new(
            move |ctx_field| {
                let inst = iovec {
                    iov_base: iov_base.into(),
                    iov_len: iov_len.into(),
                };
                let inner_ptr: *mut iovec = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            Some(Box::new(move |_ctx_field| {
                if let Some(drop_closure) = drop_copies_closure.borrow_mut().take() {
                    drop_closure();
                }
                0
            })),
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(r_constructor),
        })
    }
    #[inline]
    #[doc = r" creates zeroed struct where the underlying c struct is on the heap"]
    pub fn new_zeroed() -> Result<Self, AeronCError> {
        let resource = ManagedCResource::new(
            move |ctx_field| {
                #[cfg(debug_assertions)]
                log::debug!(
                    "creating zeroed empty resource on heap {}",
                    stringify!(iovec)
                );
                let inst: iovec = unsafe { std::mem::zeroed() };
                let inner_ptr: *mut iovec = Box::into_raw(Box::new(inst));
                unsafe { *ctx_field = inner_ptr };
                0
            },
            None,
            true,
            None,
        )?;
        Ok(Self {
            inner: std::rc::Rc::new(resource),
        })
    }
    #[inline]
    pub fn iov_base(&self) -> *mut ::std::os::raw::c_void {
        self.iov_base.into()
    }
    #[inline]
    pub fn iov_len(&self) -> usize {
        self.iov_len.into()
    }
    #[inline(always)]
    pub fn get_inner(&self) -> *mut iovec {
        self.inner.get()
    }
}
impl std::ops::Deref for Iovec {
    type Target = iovec;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}
impl From<*mut iovec> for Iovec {
    #[inline]
    fn from(value: *mut iovec) -> Self {
        Iovec {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<Iovec> for *mut iovec {
    #[inline]
    fn from(value: Iovec) -> Self {
        value.get_inner()
    }
}
impl From<&Iovec> for *mut iovec {
    #[inline]
    fn from(value: &Iovec) -> Self {
        value.get_inner()
    }
}
impl From<Iovec> for iovec {
    #[inline]
    fn from(value: Iovec) -> Self {
        unsafe { *value.get_inner().clone() }
    }
}
impl From<*const iovec> for Iovec {
    #[inline]
    fn from(value: *const iovec) -> Self {
        Iovec {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(value, None)),
        }
    }
}
impl From<iovec> for Iovec {
    #[inline]
    fn from(mut value: iovec) -> Self {
        Iovec {
            inner: std::rc::Rc::new(ManagedCResource::new_borrowed(
                &mut value as *mut iovec,
                None,
            )),
        }
    }
}
#[doc = r" This will create an instance where the struct is zeroed, use with care"]
impl Default for Iovec {
    fn default() -> Self {
        Iovec::new_zeroed().expect("failed to create struct")
    }
}
impl Iovec {
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
        copy.inner.get_mut().clone_from(self.deref());
        copy
    }
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronBroadcastReceiverHandlerCallback {
    fn handle_aeron_broadcast_receiver_handler(&mut self, type_id: i32, buffer: &mut [u8]) -> ();
}
pub struct AeronBroadcastReceiverHandlerLogger;
impl AeronBroadcastReceiverHandlerCallback for AeronBroadcastReceiverHandlerLogger {
    fn handle_aeron_broadcast_receiver_handler(&mut self, type_id: i32, buffer: &mut [u8]) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_broadcast_receiver_handler),
            [
                format!("{} : {:?}", stringify!(type_id), type_id),
                format!("{} : {:?}", stringify!(buffer), buffer)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronBroadcastReceiverHandlerLogger {}
unsafe impl Sync for AeronBroadcastReceiverHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_broadcast_receiver_handler_handler(
    ) -> Option<&'static Handler<AeronBroadcastReceiverHandlerLogger>> {
        None::<&Handler<AeronBroadcastReceiverHandlerLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_broadcast_receiver_handler_t_callback<
    F: AeronBroadcastReceiverHandlerCallback,
>(
    type_id: i32,
    buffer: *mut u8,
    length: usize,
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
            stringify!(handle_aeron_broadcast_receiver_handler)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure.handle_aeron_broadcast_receiver_handler(type_id.into(), unsafe {
        if buffer.is_null() {
            &mut [] as &mut [_]
        } else {
            std::slice::from_raw_parts_mut(buffer, length.try_into().unwrap())
        }
    })
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_broadcast_receiver_handler_t_callback_for_once_closure<
    F: FnMut(i32, &mut [u8]) -> (),
>(
    type_id: i32,
    buffer: *mut u8,
    length: usize,
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
            stringify!(aeron_broadcast_receiver_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(clientd as *mut F);
    closure(type_id.into(), unsafe {
        if buffer.is_null() {
            &mut [] as &mut [_]
        } else {
            std::slice::from_raw_parts_mut(buffer, length.try_into().unwrap())
        }
    })
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronRbHandlerCallback {
    fn handle_aeron_rb_handler(
        &mut self,
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
    ) -> ();
}
pub struct AeronRbHandlerLogger;
impl AeronRbHandlerCallback for AeronRbHandlerLogger {
    fn handle_aeron_rb_handler(
        &mut self,
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
    ) -> () {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_rb_handler),
            [
                format!("{} : {:?}", stringify!(arg1), arg1),
                format!("{} : {:?}", stringify!(arg2), arg2),
                format!("{} : {:?}", stringify!(arg3), arg3)
            ]
            .join(",\n\t"),
        );
        ()
    }
}
unsafe impl Send for AeronRbHandlerLogger {}
unsafe impl Sync for AeronRbHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_rb_handler_handler() -> Option<&'static Handler<AeronRbHandlerLogger>> {
        None::<&Handler<AeronRbHandlerLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_rb_handler_t_callback<F: AeronRbHandlerCallback>(
    arg1: i32,
    arg2: *const ::std::os::raw::c_void,
    arg3: usize,
    arg4: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if arg4.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_rb_handler));
    }
    let closure: &mut F = &mut *(arg4 as *mut F);
    closure.handle_aeron_rb_handler(arg1.into(), arg2.into(), arg3.into())
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_rb_handler_t_callback_for_once_closure<
    F: FnMut(i32, *const ::std::os::raw::c_void, usize) -> (),
>(
    arg1: i32,
    arg2: *const ::std::os::raw::c_void,
    arg3: usize,
    arg4: *mut ::std::os::raw::c_void,
) -> () {
    #[cfg(debug_assertions)]
    if arg4.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_rb_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(arg4 as *mut F);
    closure(arg1.into(), arg2.into(), arg3.into())
}
#[doc = r""]
#[doc = r""]
#[doc = r" _(note you must copy any arguments that you use afterwards even those with static lifetimes)_"]
pub trait AeronRbControlledHandlerCallback {
    fn handle_aeron_rb_controlled_handler(
        &mut self,
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
    ) -> aeron_rb_read_action_t;
}
pub struct AeronRbControlledHandlerLogger;
impl AeronRbControlledHandlerCallback for AeronRbControlledHandlerLogger {
    fn handle_aeron_rb_controlled_handler(
        &mut self,
        arg1: i32,
        arg2: *const ::std::os::raw::c_void,
        arg3: usize,
    ) -> aeron_rb_read_action_t {
        log::info!(
            "{}(\n\t{}\n)",
            stringify!(handle_aeron_rb_controlled_handler),
            [
                format!("{} : {:?}", stringify!(arg1), arg1),
                format!("{} : {:?}", stringify!(arg2), arg2),
                format!("{} : {:?}", stringify!(arg3), arg3)
            ]
            .join(",\n\t"),
        );
        unimplemented!()
    }
}
unsafe impl Send for AeronRbControlledHandlerLogger {}
unsafe impl Sync for AeronRbControlledHandlerLogger {}
impl Handlers {
    #[doc = r" No handler is set i.e. None with correct type"]
    pub fn no_rb_controlled_handler_handler(
    ) -> Option<&'static Handler<AeronRbControlledHandlerLogger>> {
        None::<&Handler<AeronRbControlledHandlerLogger>>
    }
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_rb_controlled_handler_t_callback<F: AeronRbControlledHandlerCallback>(
    arg1: i32,
    arg2: *const ::std::os::raw::c_void,
    arg3: usize,
    arg4: *mut ::std::os::raw::c_void,
) -> aeron_rb_read_action_t {
    #[cfg(debug_assertions)]
    if arg4.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!("calling {}", stringify!(handle_aeron_rb_controlled_handler));
    }
    let closure: &mut F = &mut *(arg4 as *mut F);
    closure.handle_aeron_rb_controlled_handler(arg1.into(), arg2.into(), arg3.into())
}
#[allow(dead_code)]
unsafe extern "C" fn aeron_rb_controlled_handler_t_callback_for_once_closure<
    F: FnMut(i32, *const ::std::os::raw::c_void, usize) -> aeron_rb_read_action_t,
>(
    arg1: i32,
    arg2: *const ::std::os::raw::c_void,
    arg3: usize,
    arg4: *mut ::std::os::raw::c_void,
) -> aeron_rb_read_action_t {
    #[cfg(debug_assertions)]
    if arg4.is_null() {
        unimplemented!("closure should not be null")
    }
    #[cfg(feature = "extra-logging")]
    {
        log::debug!(
            "calling {}",
            stringify!(aeron_rb_controlled_handler_t_callback_for_once_closure)
        );
    }
    let closure: &mut F = &mut *(arg4 as *mut F);
    closure(arg1.into(), arg2.into(), arg3.into())
}

