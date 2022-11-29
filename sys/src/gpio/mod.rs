pub mod fake;

#[allow(unused)]
pub mod trace {
    use aa_foundation::trace_category;
    trace_category!("gpio");
    pub(crate) use {debug, error, info, trace, warning};
}
