use std::fmt;

use nu_ansi_term::{Color, Style};

pub(super) struct FmtTarget<'a> {
    target: &'a str,
    min_width: usize,
    background: Option<Color>,
}

impl<'a> FmtTarget<'a> {
    pub(super) fn new(target: &'a str, min_width: usize, background: Option<Color>) -> Self {
        Self {
            target,
            min_width,
            background,
        }
    }

    #[inline(always)]
    fn target_to_display(&self) -> (&'a str, bool) {
        let target = self.target;
        if target.len() + 2 > self.min_width {
            for (i, _) in target.rmatch_indices("::") {
                return (&target[(i + 2)..], true);
            }
        }

        (target, false)
    }
}

impl<'a> fmt::Display for FmtTarget<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let color = self.background.unwrap_or(Color::Default);
        let structure_style = Style::new().dimmed();
        let target_name_style = Style::new().bold().on(color);
        let (target, show_leading_colons) = self.target_to_display();
        let leading_colons = if show_leading_colons { "::" } else { "" };

        write!(
            f,
            "{}{}{}{}",
            structure_style.paint("["),
            structure_style.paint(leading_colons),
            target_name_style.paint(target),
            structure_style.paint("]")
        )?;
        let spaces = self.min_width - (target.len() + 2);
        for _ in 0..spaces {
            write!(f, " ")?;
        }
        Ok(())
    }
}
