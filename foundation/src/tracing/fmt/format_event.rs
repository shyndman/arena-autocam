use std::fmt;

use nu_ansi_term::Style;
use palette::Srgba;
use tracing::field::Visit;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields, Writer};
// use tracing_subscriber::fmt::Rec
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::{self, LookupSpan};
use valuable::Value;

use super::context::FmtCtx;
use super::level::FmtLevel;
use super::target::FmtTarget;
use super::thread::FmtThreadName;
use crate::tracing::category::TARGET_VALUE_TYPE_NAME;

#[derive(Default)]
pub struct PrettyFormatter<T> {
    pub(crate) timer: T,
    pub(crate) display_timestamp: bool,
    pub(crate) target_max_len: usize,
    pub(crate) display_target: bool,
    pub(crate) display_thread_id: bool,
    pub(crate) display_thread_name: bool,
    pub(crate) display_filename: bool,
    pub(crate) display_line_number: bool,
}

impl<T> PrettyFormatter<T> {
    #[inline]
    fn format_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result
    where
        T: FormatTime,
    {
        // If timestamps are disabled, do nothing.
        if !self.display_timestamp {
            return Ok(());
        }

        // If ANSI color codes are enabled, format the timestamp with ANSI
        // colors.
        if writer.has_ansi_escapes() {
            let style = Style::new().dimmed();
            write!(writer, "{}", style.prefix())?;

            // If getting the timestamp failed, don't bail --- only bail on
            // formatting errors.
            if self.timer.format_time(writer).is_err() {
                writer.write_str("<unknown time>")?;
            }

            write!(writer, "{} ", style.suffix())?;
            return Ok(());
        }

        // Otherwise, just format the timestamp without ANSI formatting.
        // If getting the timestamp failed, don't bail --- only bail on
        // formatting errors.
        if self.timer.format_time(writer).is_err() {
            writer.write_str("<unknown time>")?;
        }
        writer.write_char(' ')
    }

    #[inline]
    fn get_target_fmt<'a>(&self, event: &'a Event<'a>) -> Option<Srgba<u8>> {
        let mut v = VisitTargetFmt::default();
        event.record(&mut v);
        v.background
    }
}

#[derive(Default)]
struct VisitTargetFmt {
    background: Option<Srgba<u8>>,
}
impl Visit for VisitTargetFmt {
    fn record_value(&mut self, _: &tracing_core::Field, value: valuable::Value<'_>) {
        if let valuable::Value::Structable(obj) = value {
            if obj.definition().name() == TARGET_VALUE_TYPE_NAME {
                obj.visit(self);
            }
        };
    }

    fn record_debug(&mut self, _: &tracing_core::Field, _: &dyn fmt::Debug) {}
}
impl valuable::Visit for VisitTargetFmt {
    fn visit_value(&mut self, value: Value<'_>) {
        match value {
            Value::Structable(v) => v.visit(self),
            _ => {} // do nothing for other types
        }
    }

    fn visit_named_fields(&mut self, named_values: &valuable::NamedValues<'_>) {
        for (field, value) in named_values {
            match (field.name(), value) {
                ("background", valuable::Value::Tuplable(t)) => {
                    let mut v = VisitBackground::default();
                    t.visit(&mut v);
                    self.background = v.background;
                }
                _ => {}
            };
        }
    }
}

#[derive(Default)]
struct VisitBackground {
    background: Option<Srgba<u8>>,
}

impl valuable::Visit for VisitBackground {
    fn visit_value(&mut self, value: Value<'_>) {
        if let Value::Tuplable(t) = value {
            t.visit(self);
        }
    }

    fn visit_unnamed_fields(&mut self, values: &[Value<'_>]) {
        let components: Vec<u8> = values
            .iter()
            .map(|v| {
                if let valuable::Value::U8(c) = v {
                    *c
                } else {
                    panic!()
                }
            })
            .collect();

        self.background =
            Some((components[0], components[1], components[2], components[3]).into());
    }
}

impl<S, N, T> FormatEvent<S, N> for PrettyFormatter<T>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
    T: FormatTime,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let meta = event.metadata();
        let target_fmt = self.get_target_fmt(event);

        // self.get_t

        // if the `Format` struct *also* has an ANSI color configuration,
        // override the writer...the API for configuring ANSI color codes on the
        // `Format` struct is deprecated, but we still need to honor those
        // configurations.
        // writer = writer.with_ansi(ansi);

        self.format_timestamp(&mut writer)?;

        let fmt_level = FmtLevel::new(meta.level(), writer.has_ansi_escapes());

        write!(writer, "{} ", fmt_level)?;

        if self.display_thread_name {
            let current_thread = std::thread::current();
            match current_thread.name() {
                Some(name) => {
                    write!(writer, "{} ", FmtThreadName::new(name))?;
                }
                // fall-back to thread id when name is absent and ids are not enabled
                None if !self.display_thread_id => {
                    write!(writer, "{:0>2?} ", current_thread.id())?;
                }
                _ => {}
            }
        }

        if self.display_thread_id {
            write!(writer, "{:0>2?} ", std::thread::current().id())?;
        }

        let fmt_ctx = FmtCtx::new(ctx, event.parent(), writer.has_ansi_escapes());
        write!(writer, "{}", fmt_ctx)?;

        let bold = Style::new().bold();
        let dimmed = Style::new().dimmed();

        let mut needs_space = false;
        if self.display_target {
            let fmt_target = FmtTarget::new(
                meta.target(),
                self.target_max_len,
                target_fmt.map(super::to_ansi_color),
            );
            write!(writer, "{}", fmt_target)?;
            needs_space = true;
        }

        if self.display_filename {
            if let Some(filename) = meta.file() {
                if self.display_target {
                    writer.write_char(' ')?;
                }
                write!(writer, "{}{}", bold.paint(filename), dimmed.paint(":"))?;
                needs_space = true;
            }
        }

        if self.display_line_number {
            if let Some(line_number) = meta.line() {
                write!(
                    writer,
                    "{}{}{}{}",
                    bold.prefix(),
                    line_number,
                    bold.suffix(),
                    dimmed.paint(":")
                )?;
                needs_space = true;
            }
        }

        if needs_space {
            writer.write_char(' ')?;
        }

        ctx.format_fields(writer.by_ref(), event)?;

        for span in ctx
            .event_scope()
            .into_iter()
            .flat_map(registry::Scope::from_root)
        {
            let exts = span.extensions();
            if let Some(fields) = exts.get::<FormattedFields<N>>() {
                if !fields.is_empty() {
                    write!(writer, " {}", dimmed.paint(&fields.fields))?;
                }
            }
        }
        writeln!(writer)
    }
}
