use color_eyre::SectionExt;
use color_eyre::{eyre::eyre, Help, Result};
use nom::{
    branch::*,
    bytes::streaming::*,
    character::streaming::*,
    combinator::*,
    error::{ErrorKind, ParseError},
    multi::*,
    sequence::*,
    IResult,
};

use crate::ssml_constants::*;
use crate::xml_writer::XmlWriter;

use std::collections::BTreeMap;
use std::str;

#[derive(Clone, Debug)]
pub struct StartTag {
    pub tag_key: String,
    pub params: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct EndTag {
    pub tag_key: String,
}

#[derive(Clone, Debug)]
pub struct OneItem {
    pub start_tag: Option<StartTag>,
    pub end_tag: Option<EndTag>,
    pub data: Option<String>,
}

fn string<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    alt((take_until("${"), rest))(input)
}

fn start_tag_info<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, StartTag, E> {
    let res = tuple((tag("${"), not(char('/')), take_until("}"), tag("}")))(input)?;
    let (left_input, (_, _, key, _)): (&str, (_, _, &str, _)) = res;
    let start_tag = if key.contains("|") {
        let mut as_split = key.split("|");
        let tag_key = as_split.next().unwrap().to_owned();
        let mut parsed_out_values = BTreeMap::new();
        loop {
            match as_split.next() {
                Some(x) => {
                    let mut as_split_new = x.split("=");
                    let btree_key = as_split_new.next();
                    let btree_value = as_split_new.next();
                    if btree_key.is_none() || btree_value.is_none() {
                        break;
                    }
                    parsed_out_values.insert(
                        btree_key.unwrap().to_owned(),
                        btree_value.unwrap().to_owned(),
                    );
                }
                None => break,
            };
        }
        StartTag {
            tag_key: tag_key,
            params: parsed_out_values,
        }
    } else {
        StartTag {
            tag_key: key.to_owned(),
            params: BTreeMap::new(),
        }
    };

    Ok((left_input, start_tag))
}

fn end_tag_info<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, EndTag, E> {
    let res = tuple((tag("${/"), take_until("}"), tag("}")))(input)?;
    let (left_input, (_, key, _)): (&str, (_, &str, _)) = res;
    Ok((
        left_input,
        EndTag {
            tag_key: key.to_owned(),
        },
    ))
}

fn text_to_ssml_parser<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Vec<OneItem>, E> {
    many1(complete(alt((
        map(start_tag_info, |start_tag| OneItem {
            start_tag: Some(start_tag),
            end_tag: None,
            data: None,
        }),
        map(end_tag_info, |end_tag| OneItem {
            start_tag: None,
            end_tag: Some(end_tag),
            data: None,
        }),
        map(string, |strz| OneItem {
            start_tag: None,
            end_tag: None,
            data: Some(strz.to_owned()),
        }),
    ))))(input)
}

/// Parses some text as SSML. It should note the error here allows for a lot of wiggle room.
/// It's still totally possible to generate invalid SSML with this. This simply does what the
/// user tells it too. If a user doesn't close a tag, we won't close a tag. If they close a
/// tag without opening one we won't close it. If they include a paragraph tag inside a paragraph
/// tag we'll still render it. All of these are invalid SSML, but don't trigger an error.
/// This is meant to be that way as you can try anything with SSML, since polly doesn't fully
/// follow the SSML v1.1 spec, now you can play around as much as you want.
pub fn parse_as_ssml(data: &str) -> Result<String> {
    let parsed = {
        if data.contains("${") {
            let res = text_to_ssml_parser::<(&str, ErrorKind)>(data);
            if res.is_err() {
                return Err(eyre!("Failed to parse string!"))
                    .with_section(|| format!("{:?}", res).header("Raw Error:"));
            }
            res.unwrap().1
        } else {
            vec![OneItem {
                start_tag: None,
                end_tag: None,
                data: Some(data.to_owned()),
            }]
        }
    };

    let mut xml_writer = XmlWriter::new()?;
    xml_writer.start_ssml_speak(None, None)?;

    let _ = parsed
        .into_iter()
        .inspect(|item| {
            if let Some(ref start_tag) = item.start_tag {
                let as_tag = start_tag.tag_key.clone().parse::<PossibleOpenTags>();
                if as_tag.is_err() {
                    return;
                }
                let tag_frd = as_tag.unwrap();

                match tag_frd {
                    PossibleOpenTags::Break => {
                        let mut strength: Option<BreakStrength> = None;
                        let mut time: Option<BreakTime> = None;

                        if start_tag.params.contains_key("strength") {
                            let attempted_parse = start_tag
                                .params
                                .get("strength")
                                .unwrap()
                                .parse::<BreakStrength>();
                            if attempted_parse.is_ok() {
                                strength = Some(attempted_parse.unwrap());
                            }
                        }
                        if start_tag.params.contains_key("time") {
                            let attempted_parse =
                                start_tag.params.get("time").unwrap().parse::<BreakTime>();
                            if attempted_parse.is_ok() {
                                time = Some(attempted_parse.unwrap());
                            }
                        }
                        let _ = xml_writer.ssml_break(strength, time);
                    }
                    PossibleOpenTags::LangTag => {
                        if !start_tag.params.contains_key("lang") {
                            return;
                        }
                        let lang = start_tag.params.get("lang").unwrap().to_owned();
                        let mut onlangfailure: Option<String> = None;
                        if start_tag.params.contains_key("onlangfailure") {
                            onlangfailure =
                                Some(start_tag.params.get("onlangfailure").unwrap().to_owned());
                        }
                        let _ = xml_writer.start_ssml_lang(lang, onlangfailure);
                    }
                    PossibleOpenTags::Mark => {
                        if !start_tag.params.contains_key("name") {
                            return;
                        }
                        let name = start_tag.params.get("name").unwrap().to_owned();
                        let _ = xml_writer.start_ssml_mark(name);
                    }
                    PossibleOpenTags::Paragraph => {
                        let _ = xml_writer.start_ssml_paragraph();
                    }
                    PossibleOpenTags::Phoneme => {
                        if !start_tag.params.contains_key("alphabet")
                            || !start_tag.params.contains_key("ph")
                        {
                            return;
                        }
                        let potential_alphabet = start_tag
                            .params
                            .get("alphabet")
                            .unwrap()
                            .parse::<PhonemeAlphabet>();
                        if potential_alphabet.is_err() {
                            return;
                        }
                        let alphabet = potential_alphabet.unwrap();
                        let ph = start_tag.params.get("ph").unwrap().to_owned();
                        let _ = xml_writer.start_ssml_phoneme(alphabet, ph);
                    }
                    PossibleOpenTags::Prosody => {
                        let mut volume: Option<String> = None;
                        let mut rate: Option<ProsodyRate> = None;
                        let mut pitch: Option<String> = None;

                        if start_tag.params.contains_key("volume") {
                            volume = Some(start_tag.params.get("volume").unwrap().to_owned());
                        }
                        if start_tag.params.contains_key("rate") {
                            let potentially_parsed =
                                start_tag.params.get("rate").unwrap().parse::<ProsodyRate>();
                            if potentially_parsed.is_ok() {
                                rate = Some(potentially_parsed.unwrap());
                            }
                        }
                        if start_tag.params.contains_key("pitch") {
                            pitch = Some(start_tag.params.get("pitch").unwrap().to_owned());
                        }

                        let _ = xml_writer.start_ssml_prosody(volume, rate, pitch);
                    }
                    PossibleOpenTags::Sentence => {
                        let _ = xml_writer.start_ssml_sentence();
                    }
                    PossibleOpenTags::SayAs => {
                        if !start_tag.params.contains_key("interpret-as") {
                            return;
                        }
                        let interpret_as = start_tag.params.get("interpret-as").unwrap().to_owned();
                        let _ = xml_writer.start_ssml_say_as(interpret_as);
                    }
                    PossibleOpenTags::Sub => {
                        if !start_tag.params.contains_key("alias") {
                            return;
                        }
                        let alias = start_tag.params.get("alias").unwrap().to_owned();
                        let _ = xml_writer.start_ssml_sub(alias);
                    }
                    PossibleOpenTags::Word => {
                        if !start_tag.params.contains_key("role") {
                            return;
                        }
                        let potentially_parsed =
                            start_tag.params.get("role").unwrap().parse::<WordRole>();
                        if potentially_parsed.is_ok() {
                            let _ = xml_writer.start_ssml_w(potentially_parsed.unwrap());
                        }
                    }
                    PossibleOpenTags::AmazonEffect => {
                        if !start_tag.params.contains_key("name")
                            && !start_tag.params.contains_key("vocal-tract-length")
                            && !start_tag.params.contains_key("phonation")
                        {
                            return;
                        }
                        if start_tag.params.contains_key("name") {
                            let potentially_parsed = start_tag
                                .params
                                .get("name")
                                .unwrap()
                                .parse::<AmazonEffect>();
                            if potentially_parsed.is_ok() {
                                let _ = xml_writer
                                    .start_ssml_amazon_effect(potentially_parsed.unwrap());
                            }
                        } else if start_tag.params.contains_key("vocal-tract-length") {
                            let factor = start_tag.params.get("vocal-tract-length").unwrap();
                            let _ = xml_writer.start_ssml_vocal_tract_length(factor.to_owned());
                        } else {
                            let potentially_parsed = start_tag
                                .params
                                .get("phonation")
                                .unwrap()
                                .parse::<PhonationVolume>();
                            if potentially_parsed.is_ok() {
                                let _ =
                                    xml_writer.start_ssml_phonation(potentially_parsed.unwrap());
                            }
                        }
                    }
                    PossibleOpenTags::AmazonAutoBreaths => {
                        let volume = start_tag
                            .params
                            .get("volume")
                            .unwrap_or(&"".to_owned())
                            .parse::<BreathVolumes>();
                        let frequency = start_tag
                            .params
                            .get("frequency")
                            .unwrap_or(&"".to_owned())
                            .parse::<AutoBreathFrequency>();
                        let duration = start_tag
                            .params
                            .get("duration")
                            .unwrap_or(&"".to_owned())
                            .parse::<BreathDuration>();

                        if volume.is_ok() && frequency.is_ok() && duration.is_ok() {
                            let _ = xml_writer.start_ssml_auto_breaths(
                                volume.unwrap(),
                                frequency.unwrap(),
                                duration.unwrap(),
                            );
                        }
                    }
                    PossibleOpenTags::AmazonBreath => {
                        let volume = start_tag
                            .params
                            .get("volume")
                            .unwrap_or(&"".to_owned())
                            .parse::<BreathVolumes>();
                        let duration = start_tag
                            .params
                            .get("duration")
                            .unwrap_or(&"".to_owned())
                            .parse::<BreathDuration>();

                        if volume.is_ok() && duration.is_ok() {
                            let _ =
                                xml_writer.write_amazon_breath(volume.unwrap(), duration.unwrap());
                        }
                    }
                    PossibleOpenTags::AmazonDomain => {
                        let name = start_tag
                            .params
                            .get("name")
                            .unwrap_or(&"".to_owned())
                            .parse::<AmazonDomainNames>();

                        if name.is_ok() {
                            let _ = xml_writer.start_ssml_amazon_domain(name.unwrap());
                        }
                    }
                };
            };

            if let Some(ref end_tag) = item.end_tag {
                let as_tag = end_tag.tag_key.clone().parse::<PossibleClosingTags>();
                if as_tag.is_err() {
                    return;
                }
                let tag_frd = as_tag.unwrap();

                let _ = match tag_frd {
                    PossibleClosingTags::LangTag => xml_writer.end_ssml_lang(),
                    PossibleClosingTags::Mark => xml_writer.end_ssml_mark(),
                    PossibleClosingTags::Paragraph => xml_writer.end_ssml_paragraph(),
                    PossibleClosingTags::Phoneme => xml_writer.end_ssml_phoneme(),
                    PossibleClosingTags::Prosody => xml_writer.end_ssml_prosody(),
                    PossibleClosingTags::Sentence => xml_writer.end_ssml_sentence(),
                    PossibleClosingTags::SayAs => xml_writer.end_ssml_say_as(),
                    PossibleClosingTags::Sub => xml_writer.end_ssml_sub(),
                    PossibleClosingTags::Word => xml_writer.end_ssml_w(),
                    PossibleClosingTags::AmazonEffect => xml_writer.end_ssml_amazon_effect(),
                    PossibleClosingTags::AmazonAutoBreaths => {
                        xml_writer.end_ssml_amazon_auto_breaths()
                    }
                    PossibleClosingTags::AmazonDomain => xml_writer.end_ssml_amazon_domain(),
                };
            };

            if let Some(ref data) = item.data {
                let _ = xml_writer.write_text(data.replace("$\\{", "${").as_str());
            }
        })
        .count();

    xml_writer.end_ssml_speak()?;

    Ok(xml_writer.render())
}
