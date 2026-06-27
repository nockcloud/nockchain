use std::collections::*;
use std::sync::Arc;

use chumsky::input::{Stream, ValueInput};
use chumsky::prelude::*;

use crate::ast::hoon::*;
use crate::utils::*;

pub fn mic_runes_tall<'src>(
    hoon: impl ParserExt<'src, Hoon>,
    spec: impl ParserExt<'src, Spec>,
    linemap: Arc<LineMap>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    choice((
        just(':').ignore_then(miccol(hoon.clone())),
        just("/").ignore_then(micfas(hoon.clone())),
        just("<").ignore_then(micgal(hoon.clone(), spec.clone())),
        just('~').ignore_then(micsig(hoon.clone(), linemap.clone())),
        just(";").ignore_then(micmic(hoon.clone(), spec.clone())),
    ))
}

pub fn mic_runes_wide<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
    spec_wide: impl ParserExt<'src, Spec>,
    linemap: Arc<LineMap>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    choice((
        just(':').ignore_then(miccol_wide(hoon_wide.clone())),
        just("/").ignore_then(micfas_wide(hoon_wide.clone())),
        just("<").ignore_then(micgal_wide(hoon_wide.clone(), spec_wide.clone())),
        just('~').ignore_then(micsig_wide(hoon_wide.clone(), linemap.clone())),
        just(";").ignore_then(micmic_wide(hoon_wide.clone(), spec_wide.clone())),
    ))
}

fn attach_rune_help_to_hoon(hoon: Hoon, help: NounExpr) -> Hoon {
    if hoon_tail_has_help(&hoon, &help) {
        return hoon;
    }
    match hoon {
        Hoon::Dbug(spot, inner) => Hoon::Dbug(spot, Box::new(Hoon::Note(Note::Help(help), inner))),
        other => Hoon::Note(Note::Help(help), Box::new(other)),
    }
}

fn with_rune_help(hoon: Hoon, start: usize, end: usize, linemap: &LineMap) -> Hoon {
    if let Some(help) = linemap.help_after_rune(start, end) {
        attach_rune_help_to_hoon(hoon, help)
    } else {
        hoon
    }
}

pub fn micsig<'src>(
    hoon: impl ParserExt<'src, Hoon>,
    linemap: Arc<LineMap>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    gap()
        .ignore_then(hoon.clone().map_with(|func: Hoon, e| {
            (func, e.span().start(), e.span().end())
        }))
        .then_ignore(gap())
        .then(
            hoon.clone()
                .map_with(|arg: Hoon, e| (arg, e.span().start(), e.span().end()))
                .separated_by(gap())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .then_ignore(gap())
        .then_ignore(just("=="))
        .map(move |((func, start, end), args)| {
            let func = with_rune_help(func, start, end, linemap.as_ref());
            let args = args
                .into_iter()
                .map(|(arg, start, end)| with_rune_help(arg, start, end, linemap.as_ref()))
                .collect();
            Hoon::MicSig(Box::new(func), args)
        })
}

pub fn micsig_wide<'src>(
    hoon: impl ParserExt<'src, Hoon>,
    linemap: Arc<LineMap>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    hoon.clone()
        .map_with(|func: Hoon, e| (func, e.span().start(), e.span().end()))
        .then(
            just(' ')
                .ignore_then(hoon.clone())
                .repeated()
                .collect::<Vec<_>>(),
        )
        .delimited_by(just('('), just(')'))
        .map(move |((func, start, end), args)| {
            let func = with_rune_help(func, start, end, linemap.as_ref());
            Hoon::MicSig(Box::new(func), args)
        })
}

pub fn micmic_wide<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
    spec_wide: impl ParserExt<'src, Spec>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    spec_wide
        .clone()
        .then_ignore(just(' '))
        .then(hoon_wide.clone())
        .delimited_by(just('('), just(')'))
        .map(|(s, h)| Hoon::MicMic(Box::new(s), Box::new(h)))
}

pub fn micgal<'src>(
    hoon: impl ParserExt<'src, Hoon>,
    spec: impl ParserExt<'src, Spec>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    gap()
        .ignore_then(spec.clone())
        .then(three_hoons_tall(hoon.clone()))
        .map(|(p, ((q, r), s))| Hoon::MicGal(Box::new(p), Box::new(q), Box::new(r), Box::new(s)))
}

pub fn micgal_wide<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
    spec_wide: impl ParserExt<'src, Spec>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    spec_wide
        .clone()
        .then_ignore(just(' '))
        .then(three_hoons_wide(hoon_wide.clone()))
        .delimited_by(just('('), just(')'))
        .map(|(p, ((q, r), s))| Hoon::MicGal(Box::new(p), Box::new(q), Box::new(r), Box::new(s)))
}

pub fn micmic<'src>(
    hoon: impl ParserExt<'src, Hoon>,
    spec: impl ParserExt<'src, Spec>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    gap()
        .ignore_then(spec.clone())
        .then_ignore(gap())
        .then(hoon.clone())
        .map(|(s, h)| Hoon::MicMic(Box::new(s), Box::new(h)))
}

pub fn micfas<'src>(
    hoon: impl ParserExt<'src, Hoon>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    gap()
        .ignore_then(hoon.clone())
        .map(|(h)| Hoon::MicFas(Box::new(h)))
}

pub fn micfas_wide<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    hoon_wide
        .clone()
        .delimited_by(just('('), just(')'))
        .map(|(h)| Hoon::MicFas(Box::new(h)))
}

pub fn miccol<'src>(
    hoon: impl ParserExt<'src, Hoon>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    gap()
        .ignore_then(hoon.clone())
        .then_ignore(gap())
        .then(list_hoon_tall(hoon.clone()))
        .then_ignore(gap())
        .then_ignore(just("=="))
        .map(|(p, list)| Hoon::MicCol(Box::new(p), list))
}

pub fn miccol_wide<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    hoon_wide
        .clone()
        .then_ignore(just(' '))
        .then(list_hoon_wide(hoon_wide.clone()))
        .delimited_by(just('('), just(')'))
        .map(|(p, list)| Hoon::MicCol(Box::new(p), list))
}

pub fn miccol_irregular<'src>(
    hoon_wide: impl ParserExt<'src, Hoon>,
) -> impl Parser<'src, &'src str, Hoon, Err<'src>> {
    hoon_wide
        .clone()
        .then_ignore(just(' '))
        .then(list_hoon_wide(hoon_wide.clone()))
        .delimited_by(just('('), just(')'))
        .map(|(p, list)| Hoon::MicCol(Box::new(p), list))
}
