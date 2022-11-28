pub mod fake;

#[allow(unused)]
pub mod trace {
    use aa_foundation::trace_macros_for_target;
    trace_macros_for_target!("gpio");
    pub(crate) use {debug, error, info, trace, warning};
}
