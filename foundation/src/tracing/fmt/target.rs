use std::fmt;

use nu_ansi_term::Style;

pub(super) struct FmtTarget<'a> {
    target: &'a str,
    min_width: usize,
}

impl<'a> FmtTarget<'a> {
    pub(super) fn new(target: &'a str, min_width: usize) -> Self {
        Self { target, min_width }
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
        let dimmed = Style::new().dimmed();
        let (target, show_leading_colons) = self.target_to_display();
        let leading_colons = if show_leading_colons { "::" } else { "" };
        write!(
            f,
            "{}{}{}{}",
            dimmed.paint("["),
            dimmed.paint(leading_colons),
            dimmed.paint(target),
            dimmed.paint("]")
        )?;
        let spaces = self.min_width - (target.len() + 2);
        for _ in 0..spaces {
            write!(f, " ")?;
        }
        Ok(())
    }
}
