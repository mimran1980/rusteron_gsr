use libaeron_driver_sys as aeron_driver;

use std::ffi::CStr;

use libaeron_driver_sys::aeron_driver_context_t;
use rusteron_common::ManagedCResource;

pub struct AeronContext {
    resource: ManagedCResource<aeron_driver_context_t>,
}

impl AeronContext {
    pub fn new() -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |ctx| unsafe { aeron_driver::aeron_driver_context_init(ctx) },
            |ctx| unsafe { aeron_driver::aeron_driver_context_close(ctx) },
        )
        .map_err(|error_code| {
            format!("failed to initialise aeron context error code {error_code}")
        })?;

        Ok(Self { resource })
    }

    // Add methods specific to AeronContext
    pub fn print_config(&self) -> rusteron_common::Result<(), Box<dyn std::error::Error>> {
        print_aeron_config(self.resource.get())?;
        Ok(())
    }
}

pub struct AeronDriver {
    resource: ManagedCResource<aeron_driver::aeron_driver_t>,
}

impl AeronDriver {
    pub fn new(context: &AeronContext) -> rusteron_common::Result<Self, Box<dyn std::error::Error>> {
        let resource = ManagedCResource::new(
            |driver| unsafe { aeron_driver::aeron_driver_init(driver, context.resource.get()) },
            |driver| unsafe { aeron_driver::aeron_driver_close(driver) },
        )
        .map_err(|error_code| {
            format!("failed to initialise aeron driver error code {error_code}")
        })?;

        Ok(Self { resource })
    }

    pub fn start(&self) -> rusteron_common::Result<(), Box<dyn std::error::Error>> {
        let result = unsafe { aeron_driver::aeron_driver_start(self.resource.get(), false) };
        if result < 0 {
            return Err(format!("failed to start aeron driver error code {result}").into());
        }
        Ok(())
    }

    // Add methods specific to AeronDriver
    pub fn do_work(&self) {
        while unsafe { aeron_driver::aeron_driver_main_do_work(self.resource.get()) } != 0 {
            // busy spin
        }
    }
}

fn threading_mode_to_str(mode: aeron_driver::aeron_threading_mode_t) -> &'static str {
    match mode {
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_DEDICATED => "DEDICATED",
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_SHARED_NETWORK => {
            "SHARED_NETWORK"
        }
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_SHARED => "SHARED",
        aeron_driver::aeron_threading_mode_enum::AERON_THREADING_MODE_INVOKER => "INVOKER",
        _ => "UNKNOWN",
    }
}

fn print_aeron_config(context: *mut aeron_driver::aeron_driver_context_t) -> rusteron_common::Result<()> {
    let config_entries = vec![
        (
            "dir",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_dir(context))
            }),
        ),
        (
            "dir_warn_if_exists",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_warn_if_exists(context)
            }),
        ),
        (
            "threading_mode",
            threading_mode_to_str(unsafe {
                aeron_driver::aeron_driver_context_get_threading_mode(context)
            })
            .to_string(),
        ),
        (
            "dir_delete_on_start",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_delete_on_start(context)
            }),
        ),
        (
            "dir_delete_on_shutdown",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_dir_delete_on_shutdown(context)
            }),
        ),
        (
            "to_conductor_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_to_conductor_buffer_length(context)
            }),
        ),
        (
            "to_clients_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_to_clients_buffer_length(context)
            }),
        ),
        (
            "counters_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_counters_buffer_length(context)
            }),
        ),
        (
            "error_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_error_buffer_length(context)
            }),
        ),
        (
            "client_liveness_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_client_liveness_timeout_ns(context)
            }),
        ),
        (
            "term_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_term_buffer_length(context)
            }),
        ),
        (
            "ipc_term_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_term_buffer_length(context)
            }),
        ),
        (
            "term_buffer_sparse_file",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_term_buffer_sparse_file(context)
            }),
        ),
        (
            "perform_storage_checks",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_perform_storage_checks(context)
            }),
        ),
        (
            "low_file_store_warning_threshold",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_low_file_store_warning_threshold(context)
            }),
        ),
        (
            "spies_simulate_connection",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_spies_simulate_connection(context)
            }),
        ),
        (
            "file_page_size",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_file_page_size(context)
            }),
        ),
        (
            "mtu_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_mtu_length(context)
            }),
        ),
        (
            "ipc_mtu_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_mtu_length(context)
            }),
        ),
        (
            "ipc_publication_term_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_ipc_publication_term_window_length(context)
            }),
        ),
        (
            "publication_term_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_term_window_length(context)
            }),
        ),
        (
            "publication_linger_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_linger_timeout_ns(context)
            }),
        ),
        (
            "socket_so_rcvbuf",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_so_rcvbuf(context)
            }),
        ),
        (
            "socket_so_sndbuf",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_so_sndbuf(context)
            }),
        ),
        (
            "socket_multicast_ttl",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_socket_multicast_ttl(context)
            }),
        ),
        (
            "send_to_status_poll_ratio",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_send_to_status_poll_ratio(context)
            }),
        ),
        (
            "rcv_status_message_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_rcv_status_message_timeout_ns(context)
            }),
        ),
        (
            "multicast_flowcontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_multicast_flowcontrol_supplier(context)
            }),
        ),
        (
            "unicast_flowcontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_unicast_flowcontrol_supplier(context)
            }),
        ),
        (
            "image_liveness_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_image_liveness_timeout_ns(context)
            }),
        ),
        (
            "rcv_initial_window_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_rcv_initial_window_length(context)
            }),
        ),
        (
            "congestioncontrol_supplier",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_congestioncontrol_supplier(context)
            }),
        ),
        (
            "loss_report_buffer_length",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_loss_report_buffer_length(context)
            }),
        ),
        (
            "publication_unblock_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_unblock_timeout_ns(context)
            }),
        ),
        (
            "publication_connection_timeout_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_publication_connection_timeout_ns(context)
            }),
        ),
        (
            "timer_interval_ns",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_timer_interval_ns(context)
            }),
        ),
        (
            "sender_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_sender_idle_strategy(
                    context,
                ))
            }),
        ),
        (
            "conductor_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_conductor_idle_strategy(context),
                )
            }),
        ),
        (
            "receiver_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_receiver_idle_strategy(context),
                )
            }),
        ),
        (
            "sharednetwork_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(
                    aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy(context),
                )
            }),
        ),
        (
            "shared_idle_strategy",
            format!("{:?}", unsafe {
                CStr::from_ptr(aeron_driver::aeron_driver_context_get_shared_idle_strategy(
                    context,
                ))
            }),
        ),
        (
            "sender_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_sender_idle_strategy_init_args(context)
            }),
        ),
        (
            "conductor_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_conductor_idle_strategy_init_args(context)
            }),
        ),
        (
            "receiver_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_receiver_idle_strategy_init_args(context)
            }),
        ),
        (
            "sharednetwork_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_sharednetwork_idle_strategy_init_args(
                    context,
                )
            }),
        ),
        (
            "shared_idle_strategy_init_args",
            format!("{:?}", unsafe {
                aeron_driver::aeron_driver_context_get_shared_idle_strategy_init_args(context)
            }),
        ),
    ];

    // Find the maximum length of the keys
    let max_key_len = config_entries
        .iter()
        .map(|(key, _)| key.len() + 2)
        .max()
        .unwrap_or(0);

    // Print the aligned configuration entries
    for (key, value) in config_entries {
        println!("{:width$}: {}", key, value, width = max_key_len);
    }

    println!();

    Ok(())
}
