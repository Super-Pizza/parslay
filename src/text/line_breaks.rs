#![allow(clippy::if_same_then_else)]

#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum Break {
    No,
    Maybe,
    Yes,
}

const BK: [char; 4] = ['\x0b', '\x0c', '\u{2028}', '\u{2029}'];

// Incomplete.
const BA: [char; 26] = [
    '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}', '\u{2006}',
    '\u{2008}', '\u{2009}', '\u{200a}', '\u{205f}', '\u{3000}', '\x09', '\u{00ad}', '\u{058a}',
    '\u{2010}', '\u{2012}', '\u{2013}', '\u{05be}', '\u{0f0b}', '\u{1361}', '\u{17d8}', '\u{17da}',
    '\u{2027}', '\x7c',
];

const CL: [char; 11] = [
    '\u{3001}', '\u{3002}', '\u{fe10}', '\u{fe11}', '\u{fe12}', '\u{fe50}', '\u{fe52}', '\u{ff0c}',
    '\u{ff0e}', '\u{ff61}', '\u{ff64}',
];

const EX: [char; 11] = [
    '\x21', '\x3f', '\u{05c6}', '\u{061b}', '\u{061e}', '\u{061f}', '\u{06d4}', '\u{07f9}',
    '\u{0f0d}', '\u{ff01}', '\u{ff1f}',
];

const GL: [char; 6] = [
    '\u{00a0}', '\u{202f}', '\u{180e}', '\u{034f}', '\u{2007}', '\u{2011}',
];

// List allowed breaks, after every character. Incomplete.
pub(crate) fn line_breaks(string: &str) -> Vec<Break> {
    let mut output = vec![];
    let mut iter = string.chars().peekable();
    'main: while let Some(c) = iter.next() {
        let Some(&next) = iter.peek() else {
            break 'main;
        };
        let should = if BK.contains(&c) {
            Break::Yes
        } else if c == '\r' && next == '\n' {
            Break::No
        } else if ['\x0d', '\x0a', '\u{0085}'].contains(&c) {
            Break::Yes
        } else if ['\x0d', '\x0a', '\u{0085}'].contains(&next) || BK.contains(&next) {
            Break::No
        } else if next == ' ' || next == '\u{200B}' {
            Break::No
        } else if c == '\u{200B}' {
            loop {
                match iter.peek() {
                    None => break 'main,
                    Some(&' ') => break Break::Maybe,
                    _ => {}
                }
                if iter.next().is_none() {
                    break 'main;
                }
            }
        } else if c == '\u{200D}' {
            Break::No
        } else {
            let last = *output.last().unwrap_or(&Break::No);
            let mut next = next;
            while matches!(next, '\u{0300}'..='\u{036f}' | '\u{1AB0}'..='\u{1AFF}' | '\u{1dc0}'..='\u{1dff}' | '\u{20d0}'..='\u{20ff}' | '\u{2de0}'..='\u{2dff}' | '\u{fe20}'..='\u{fe2f}' | '\u{200d}')
            {
                iter.next().unwrap();
                next = match iter.peek() {
                    None => break 'main,
                    Some(&x) => x,
                };
                output.push(Break::No);
            }
            let c = if last == Break::Yes { 'a' } else { c };
            if c == '\u{2060}' || next == '\u{2060}' {
                Break::No
            } else if GL.contains(&c) {
                Break::No
            } else if GL.contains(&next) && c != ' ' && c != '-' && !BA.contains(&c) {
                Break::No
            } else if next == '/'
                || CL.contains(&next)
                || matches!(
                    next,
                    '\x29' | '\x5d' | '\u{2e56}' | '\u{2e58}' | '\u{2e5a}' | '\u{2e5c}'
                )
                || EX.contains(&next)
            {
                Break::No
            }
            /*LB14-17 missing */
            else if c == ' ' {
                Break::Maybe
            }
            /* LB18-22 missing */
            else if (c.is_alphabetic() && next.is_numeric())
                || (c.is_numeric() && next.is_alphabetic())
            {
                Break::No
            }
            /*LB23a-LB27 missing */
            else if c.is_alphabetic() && next.is_alphabetic() {
                Break::No
            }
            /*LB28a-29 missing */
            else {
                Break::Maybe
            }
        };
        output.push(should);
    }
    output.push(Break::Yes);
    output
}
