/// Writes out tracing macros for a named log category:
///
/// * `trace!`
/// * `debug!`
/// * `info!`
/// * `warn!`
/// * `error!`
#[macro_export]
macro_rules! trace_category {
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
        #[allow(unused)]
        mod tracing {
            use $crate::trace_category;
            trace_category!{($) trace $target}
            trace_category!{($) debug $target}
            trace_category!{($) info $target}
            trace_category!{($) warning $target}
            trace_category!{($) error $target}
            pub(crate) use {trace, debug, info, warning, error};
        }
    };
}

#[cfg(test)]
mod test {
    use std::env::set_var;

    use crate::tracing::setup_dev_tracing_subscriber;

    trace_category!("test");

    #[test]
    fn test() {
        use self::tracing::*;

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
