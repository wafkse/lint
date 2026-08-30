//! Source comment marker support for syntax-policy lints.

use std::collections::BTreeSet;

use rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};
use rustc_lint::{EarlyContext, LintContext};
use rustc_span::{BytePos, Span};

/// A source marker found in an authored ordinary line comment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Marker {
    /// Span covering the marker text itself.
    pub span: Span,
}

/// Tracks marker locations across the source files visited by one lint pass.
#[derive(Default)]
pub struct MarkerIndex {
    /// Source-file start offsets that have already been scanned.
    scanned_files: BTreeSet<u32>,

    /// Invariant marker locations found in authored source.
    invariant_markers: Vec<Marker>,
}
impl MarkerIndex {
    /// Scan the authored source file containing `span` once.
    pub fn scan_file(&mut self, context: &EarlyContext<'_>, span: Span) {
        if span.from_expansion() {
            return;
        }

        let source_map = context.sess().source_map();
        let source_file = source_map.lookup_source_file(span.lo());
        let file_start = source_file.start_pos.0;

        match self.scanned_files.insert(file_start) {
            true => (),
            false => return,
        }

        let Some(source) = source_file.src.as_deref() else {
            return;
        };
        let mut offset = 0_usize;

        for token in tokenize(source, FrontmatterAllowed::No) {
            let token_length = token.len as usize;

            if let TokenKind::LineComment { doc_style: None } = token.kind {
                let token_end = offset + token_length;
                let comment = &source[offset..token_end];

                Self::record_markers(
                    &mut self.invariant_markers,
                    source_file.start_pos,
                    offset,
                    comment,
                    "NOTE(invariant):",
                );
            }

            offset += token_length;
        }
    }
    /// Return all invariant markers discovered in authored source.
    pub fn invariant_markers(&self) -> &[Marker] {
        &self.invariant_markers
    }

    /// Return markers from the ordinary comment block directly above `span`.
    pub fn adjacent(context: &EarlyContext<'_>, span: Span, marker_text: &str) -> Vec<Marker> {
        let source_map = context.sess().source_map();
        let Ok(location) = source_map.lookup_line(span.lo()) else {
            return Vec::new();
        };
        let Some(source) = location.sf.src.as_deref() else {
            return Vec::new();
        };
        let line_list = Self::line_list(source);
        let mut marker_list = Vec::new();
        let mut line_index = location.line;

        while line_index > 0 {
            line_index -= 1;
            let (line_offset, line) = line_list[line_index];
            let trimmed = line.trim_start();
            let ordinary_comment = trimmed.starts_with("//")
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("//!");

            match ordinary_comment {
                true => {
                    for marker_offset in line.match_indices(marker_text).map(|(offset, _)| offset) {
                        let absolute_offset = line_offset + marker_offset;
                        let span = Self::marker_span(
                            location.sf.start_pos,
                            absolute_offset,
                            marker_text.len(),
                        );

                        marker_list.push(Marker { span });
                    }
                }
                false => break,
            }
        }

        marker_list
    }
    fn record_markers(
        marker_list: &mut Vec<Marker>,
        file_start: BytePos,
        token_offset: usize,
        comment: &str,
        marker_text: &str,
    ) {
        for marker_offset in comment.match_indices(marker_text).map(|(offset, _)| offset) {
            let absolute_offset = token_offset + marker_offset;
            let span = Self::marker_span(file_start, absolute_offset, marker_text.len());

            marker_list.push(Marker { span });
        }
    }

    fn marker_span(file_start: BytePos, offset: usize, length: usize) -> Span {
        let lo = BytePos(file_start.0 + offset as u32);
        let hi = BytePos(lo.0 + length as u32);

        Span::with_root_ctxt(lo, hi)
    }

    fn line_list(source: &str) -> Vec<(usize, &str)> {
        let mut offset = 0_usize;
        let mut line_list = Vec::new();

        for line in source.split_inclusive('\n') {
            line_list.push((offset, line));
            offset += line.len();
        }

        match source.ends_with('\n') {
            true => line_list,
            false => {
                if line_list.is_empty() {
                    line_list.push((0, ""));
                }

                line_list
            }
        }
    }
}
