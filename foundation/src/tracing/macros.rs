/// Writes out preconfigured tracing macros for a named target:
///
/// * `trace!`
/// * `debug!`
/// * `info!`
/// * `warn!`
/// * `error!`
#[macro_export]
macro_rules! trace_macros_for_target {
    (($d:tt) $level:ident $target:expr) => {
        macro_rules! $level {
            ({ $d($field:tt)+ }, $d($arg:tt)+ ) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    $d($field)+,
                    $d($arg)+
                )
            );
            ($d($k:ident).+ = $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    $d($k).+ = $d($field)*
                )
            );
            (?$d($k:ident).+ = $d($field:tt)*) => (
                    $crate::tracing::base_macros::$level!(
                    target: $target,
                    ?$d($k).+ = $d($field)*
                )
            );
            (%$d($k:ident).+ = $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    %$d($k).+ = $d($field)*
                )
            );
            ($d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                     $d($k).+, $d($field)*
                )
            );
            (?$d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    ?$d($k).+,
                    $d($field)*
                )
            );
            (%$d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    %$d($k).+,
                    $d($field)*
                )
            );
            (?$d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    ?$d($k).+
                )
            );
            (%$d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    %$d($k).+
                )
            );
            ($d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    $d($k).+
                )
            );
            ($d($arg:tt)+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target,
                    $d($arg)+
                )
            );
        }
    };
    ($target:expr) => {
        trace_macros_for_target!{($) trace $target}
        trace_macros_for_target!{($) debug $target}
        trace_macros_for_target!{($) info $target}
        trace_macros_for_target!{($) warning $target}
        trace_macros_for_target!{($) error $target}
    };
}

#[cfg(test)]
mod test {
    use std::env::set_var;

    use crate::tracing::setup_dev_tracing_subscriber;

    trace_macros_for_target!("test");

    #[test]
    fn test() {
        set_var("RUST_LOG", "trace");
        setup_dev_tracing_subscriber();

        let string = "world";
        trace!("hello");
        debug!(string, "hello {string}");
        info!("hello {string}");
        warning!(?string, "hello {}", string);
        error!(str_attr = %string, "hello {}", string);
    }
}
