use palette::Srgba;

/// Writes out tracing macros for a named log category:
///
/// * `trace!`
/// * `debug!`
/// * `info!`
/// * `warn!`
/// * `error!`
#[macro_export]
macro_rules! trace_category {
    (($d:tt) $level:ident $target_name:expr, __target_fmt: $target_fmt:expr) => {
        trace_category!(($d) $level $target_name, {
            use $crate::tracing::valuable_crate::Valuable;
            Valuable::as_value(once_cell::sync::Lazy::force(&$target_fmt))
        });
    };
    (($d:tt) $level:ident $target_name:expr, $target_fmt:expr) => {
        macro_rules! $level {
            ({ $d($field:tt)+ }, $d($arg:tt)+ ) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    $d($field)+,
                    $d($arg)+
                )
            );
            ($d($k:ident).+ = $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    $d($k).+ = $d($field)*,
                )
            );
            (?$d($k:ident).+ = $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    ?$d($k).+ = $d($field)*
                )
            );
            (%$d($k:ident).+ = $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    %$d($k).+ = $d($field)*
                )
            );
            ($d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    $d($k).+, $d($field)*
                )
            );
            (?$d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    ?$d($k).+,
                    $d($field)*
                )
            );
            (%$d($k:ident).+, $d($field:tt)*) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    %$d($k).+,
                    $d($field)*
                )
            );
            (?$d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    ?$d($k).+
                )
            );
            (%$d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    %$d($k).+
                )
            );
            ($d($k:ident).+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    $d($k).+
                )
            );
            ($d($arg:tt)+) => (
                $crate::tracing::base_macros::$level!(
                    target: $target_name,
                    __TARGET_FMT=$target_fmt,
                    $d($arg)+
                )
            );
        }
    };
    ($target_name:expr) => {
        trace_category!($target_name, bg: $crate::color::TRANSPARENT);
    };
    ($target_name:expr, bg: $bg:expr) => {
        #[allow(unused)]
        mod tracing {
            use $crate::trace_category;
            use $crate::tracing::macros::TargetFmt;
            use once_cell::sync::Lazy;

            pub static TARGET_FMT: Lazy<TargetFmt> = Lazy::new(|| {
                TargetFmt {
                    name: $target_name,
                    background: $bg,
                }
            });

            trace_category!{($) trace $target_name, __target_fmt: TARGET_FMT}
            trace_category!{($) debug $target_name, __target_fmt: TARGET_FMT}
            trace_category!{($) info $target_name, __target_fmt: TARGET_FMT}
            trace_category!{($) warning $target_name, __target_fmt: TARGET_FMT}
            trace_category!{($) error $target_name, __target_fmt: TARGET_FMT}
            pub(crate) use {trace, debug, info, warning, error};
        }
    };
}

pub struct TargetFmt {
    pub name: &'static str,
    pub background: Srgba<u8>,
}

pub(crate) const TARGET_VALUE_TYPE_NAME: &str = "__TargetFmt";
static TARGET_FMT_FIELDS: &[valuable::NamedField<'static>] = &[
    valuable::NamedField::new("name"),
    valuable::NamedField::new("background"),
];
impl valuable::Structable for TargetFmt {
    fn definition(&self) -> valuable::StructDef<'_> {
        valuable::StructDef::new_static(
            TARGET_VALUE_TYPE_NAME,
            valuable::Fields::Named(TARGET_FMT_FIELDS),
        )
    }
}

impl valuable::Valuable for TargetFmt {
    fn as_value(&self) -> valuable::Value<'_> {
        valuable::Value::Structable(self)
    }
    fn visit(&self, visitor: &mut dyn valuable::Visit) {
        visitor.visit_named_fields(&valuable::NamedValues::new(
            TARGET_FMT_FIELDS,
            &[
                valuable::Valuable::as_value(&self.name),
                valuable::Valuable::as_value(&self.background.into_components()),
            ],
        ));
    }
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
