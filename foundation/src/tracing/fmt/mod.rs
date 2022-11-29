mod context;
mod level;
mod thread;

use std::fmt;

use nu_ansi_term::Style;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields, Writer};
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::fmt::{FmtContext, FormattedFields};
use tracing_subscriber::registry::{self, LookupSpan};

use self::context::FmtCtx;
use self::level::FmtLevel;
use self::thread::FmtThreadName;

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
        #[cfg(feature = "tracing-log")]
        let normalized_meta = event.normalized_metadata();
        #[cfg(feature = "tracing-log")]
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());
        #[cfg(not(feature = "tracing-log"))]
        let meta = event.metadata();

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

        let fmt_ctx = { FmtCtx::new(ctx, event.parent(), writer.has_ansi_escapes()) };
        write!(writer, "{}", fmt_ctx)?;

        let bold = Style::new().bold();
        let dimmed = Style::new().dimmed();

        let mut needs_space = false;
        if self.display_target {
            let pad_by = self.target_max_len - meta.target().len();
            write!(
                writer,
                "{}{}{}",
                dimmed.paint("["),
                dimmed.paint(meta.target()),
                dimmed.paint("]")
            )?;
            for _ in 0..pad_by {
                writer.write_char(' ')?;
            }
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
