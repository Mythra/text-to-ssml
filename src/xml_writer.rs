//! Controls writing of the XML part of SSML. This contains all low level bindings in a sense
//! to the tags. You should probably never use this directly.

use color_eyre::{eyre::eyre, Result};
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use std::io::Cursor;

use crate::ssml_constants::*;

/// An XML Writer. Used for manual manipulation of the SSML Output (which uses XML).
///
/// You should probably never use this directly, instead interacting with the parser,
/// however if you'd like to build your own parser, and just reuse the XML Rendering
/// then you'd want to use this.
pub struct XmlWriter {
    /// The XML Writer instance. The thing that actually writes the XML.
    pub writer: Writer<Cursor<Vec<u8>>>,
}

impl XmlWriter {
    /// Creates a new XML Writer. This writerr writes into a std::vec::Vec, and at any
    /// point can be turned into a string. It is your job to close all tags before rendering
    /// this. We don't close everything when you render it. You render what you put in.
    ///
    /// It should also note we automatically write the header:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// ```
    ///
    /// Upon creation of an XML Writer. This is to try, and keep as close to the W3C docs
    /// for SSML v1.1. Which you can read about
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let result = XmlWriter::new();
    /// assert!(result.is_ok());
    /// ```
    pub fn new() -> Result<XmlWriter> {
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        writer.write_event(Event::Decl(BytesDecl::new(b"1.0", None, None)))?;
        Ok(XmlWriter { writer: writer })
    }

    /// Starts an SSML <speak> tag. For AWS Polly this is the root tag, and should only have one
    /// decleration as mentioned in their docs (As of April 20th, 2017):
    ///
    /// ```text
    /// The <speak> tag is the root element of all Amazon Polly SSML text.
    /// All SSML-enhanced text to be spoken must be included within this tag.
    /// ```
    ///
    /// It should be noted although AWS Docs do not mention any attributes you can pass in
    /// to the <speak> tag, we still have an optional `lang`, and `onlangfailure` attributes
    /// to closely mirror the W3C Standard, as seen:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/).
    ///
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_speak_result = new_xml_writer.unwrap().start_ssml_speak(None, None);
    /// assert!(start_speak_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <speak xml:lang="en-US" onlangfailure="processorchoice"
    ///    xmlns="http://www.w3.org/2001/10/synthesis"
    ///    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    /// ```
    pub fn start_ssml_speak(
        &mut self,
        lang: Option<String>,
        onlangfailure: Option<String>,
    ) -> Result<()> {
        let mut elem = BytesStart::owned(b"speak".to_vec(), "speak".len());
        elem.push_attribute(("xml:lang", &*lang.unwrap_or("en-US".to_owned())));
        elem.push_attribute((
            "onlangfailure",
            &*onlangfailure.unwrap_or("processorchoice".to_owned()),
        ));
        elem.push_attribute(("xmlns", "http://www.w3.org/2001/10/synthesis"));
        elem.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <speak> tag. For AWS Polly this should be the root tag, and you
    /// should only close it when you are done.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_speak_result = new_xml_writer.unwrap().end_ssml_speak();
    /// assert!(end_speak_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </speak>
    /// ```
    pub fn end_ssml_speak(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"speak")))?)
    }

    /// Creates an SSML <break> tag. AWS Polly follows the W3C SSMLv1.1 standard for
    /// this tag.
    ///
    /// You can find the SSML <break> tag documented in the W3C's guide:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/).
    /// Although  you can find the specific implementation details on AWS's site:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#break-tag).
    ///
    /// According to the W3C 1.1 Standard both the strength, and time are optional.
    /// Though both can be used in combination.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let mut xml_writer = new_xml_writer.unwrap();
    /// let result = xml_writer.ssml_break(None, None);
    /// assert!(result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <break />
    /// ```
    ///
    /// ---
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::{BreakStrength, BreakTime};
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let mut xml_writer = new_xml_writer.unwrap();
    /// let result = xml_writer.ssml_break(Some(BreakStrength::XStrong), Some(BreakTime::new(10, true)));
    /// assert!(result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <break strength="x-strong" time="10s" />
    /// ```
    pub fn ssml_break(
        &mut self,
        strength: Option<BreakStrength>,
        time: Option<BreakTime>,
    ) -> Result<()> {
        let mut elem = BytesStart::owned(b"break".to_vec(), "break".len());

        if strength.is_some() {
            elem.push_attribute(("strength", &*format!("{}", strength.unwrap())));
        }
        if time.is_some() {
            elem.push_attribute(("time", &*format!("{}", time.unwrap())));
        }

        Ok(self.writer.write_event(Event::Empty(elem))?)
    }

    /// Starts an SSML Lang tag. The Lang tag is useful for telling say
    /// someone speaking in english that they're about to speak a french word. You can keep
    /// the overall text english, but have a mix of french words in there. Although AWS polly
    /// only documents support the `xml:lang` attribute, we also pass in `onlangfailure`
    /// which is documented inside the W3C SSML 1.1 standard which can be found:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_lang).
    ///
    /// You can find the AWS Documentation that mentions the lang tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#lang-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_lang_result = new_xml_writer.unwrap().start_ssml_lang("fr-FR".to_owned(), None);
    /// assert!(start_lang_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <lang xml:lang="fr-FR" onlangfailure="processorchoice">
    /// ```
    ///
    /// ---
    ///
    /// Rust Code:
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_lang_result = new_xml_writer.unwrap().start_ssml_lang("fr-FR".to_owned(),
    ///   Some("changevoice".to_owned()));
    /// assert!(start_lang_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <lang xml:lang="fr-FR" onlangfailure="changevoice">
    /// ```
    pub fn start_ssml_lang(&mut self, lang: String, onlangfailure: Option<String>) -> Result<()> {
        let mut elem = BytesStart::owned(b"lang".to_vec(), "lang".len());
        elem.push_attribute(("xml:lang", &*lang));
        elem.push_attribute((
            "onlangfailure",
            &*onlangfailure.unwrap_or("processorchoice".to_owned()),
        ));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <lang> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_lang_result = new_xml_writer.unwrap().end_ssml_lang();
    /// assert!(end_lang_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </lang>
    /// ```
    pub fn end_ssml_lang(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"lang")))?)
    }

    /// Starts an SSML Mark tag. Although this will make no difference in the voice
    /// of the text, this will place a marker inside the SSML Metadata returned from Polly.
    /// This can be useful if you want to perform some sort of actions on certain words.
    /// AWS Polly follows the W3C SSML v1.1 Spec here, and documentation can be found:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_mark).
    ///
    /// You can find the AWS Documentation that mentions the mark tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#custom-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_mark_result = new_xml_writer.unwrap().start_ssml_mark("animal".to_owned());
    /// assert!(start_mark_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// Mmark name="animal">
    /// ```
    pub fn start_ssml_mark(&mut self, name: String) -> Result<()> {
        let mut elem = BytesStart::owned(b"mark".to_vec(), "mark".len());
        elem.push_attribute(("name", &*name));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <mark> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_mark_result = new_xml_writer.unwrap().end_ssml_mark();
    /// assert!(end_mark_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </mark>
    /// ```
    pub fn end_ssml_mark(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"mark")))?)
    }

    /// Starts an SSML Paragraph Tag. The Paragraph Tag is useful for breaking
    /// up multiple paragraphs of text. AWS Polly follows the W3C SSML v1.1 Standard Here.
    /// As such the documentation for the paragraph tag can be found:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_paragraph).
    ///
    /// You can find the AWS Documentation that mentions the paragraph tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#p-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_p_result = new_xml_writer.unwrap().start_ssml_paragraph();
    /// assert!(start_p_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <p>
    /// ```
    pub fn start_ssml_paragraph(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::Start(BytesStart::owned(b"p".to_vec(), "p".len())))?)
    }

    /// Ends an SSML <p> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_p_result = new_xml_writer.unwrap().end_ssml_paragraph();
    /// assert!(end_p_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </p>
    /// ```
    pub fn end_ssml_paragraph(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"p")))?)
    }

    /// Starts an SSML Phoneme Tag. The Phoneme Tag is useful for custom pronunciation for words.
    /// The Phoneme Tag should really only be used on a per word/short phrase basis. You don't
    /// want to use a phoneme tag for an entire paragraph of text. The Phoneme Tag in polly
    /// has two required attributes both "ph", and "alphabet". Which deviates from the W3C Standard,
    /// which says only "ph" is required. However since Polly implements close to perfect the W3C
    /// SSML v1.1 Standard here you should still probably read their documentation on the tag:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_phoneme).
    ///
    /// You can find the AWS Documentation that mentions the phoneme tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#phoneme-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::PhonemeAlphabet;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_phoneme_result = new_xml_writer.unwrap().start_ssml_phoneme(PhonemeAlphabet::Ipa,
    ///  "d͡ʒt͡ʃΘɚoʊɛ".to_owned());
    /// assert!(start_phoneme_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <phoneme alphabet="ipa" ph="d͡ʒt͡ʃΘɚoʊɛ">
    /// ```
    pub fn start_ssml_phoneme(&mut self, alphabet: PhonemeAlphabet, ph: String) -> Result<()> {
        let mut elem = BytesStart::owned(b"phoneme".to_vec(), "phoneme".len());
        elem.push_attribute(("alphabet", &*format!("{}", alphabet)));
        elem.push_attribute(("ph", &*ph));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <phoneme> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_phoneme_result = new_xml_writer.unwrap().end_ssml_phoneme();
    /// assert!(end_phoneme_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </phoneme>
    /// ```
    pub fn end_ssml_phoneme(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"phoneme")))?)
    }

    /// Starts an SSML Prosody Tag. The prosody tag seems to be the one that derives the most
    /// from the SSML Specification. Which in some instances is fine because it makes for easier
    /// reading (e.g. +20% pitch), but in other places is kind of sad we can't do that. (e.g.
    /// things like duration). As such I'll only link to the AWS documentation.
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#prosody-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_prosody_result = new_xml_writer.unwrap().start_ssml_prosody(Some("+5db".to_owned()), None, None);
    /// assert!(start_prosody_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <prosody volume="+6db">
    /// ```
    ///
    /// ---
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::ProsodyRate;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_prosody_result = new_xml_writer.unwrap()
    ///   .start_ssml_prosody(Some("+6dB".to_owned()), Some(ProsodyRate::XFast),
    ///    Some("+100%".to_owned()));
    /// assert!(start_prosody_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <prosody volume="+6db" rate="x-fast" pitch="+100%">
    /// ```
    pub fn start_ssml_prosody(
        &mut self,
        volume: Option<String>,
        rate: Option<ProsodyRate>,
        pitch: Option<String>,
    ) -> Result<()> {
        let mut elem = BytesStart::owned(b"prosody".to_vec(), "prosody".len());
        if volume.is_none() && rate.is_none() && pitch.is_none() {
            return Err(eyre!("Prosody Tag was supplied no values."));
        }
        if volume.is_some() {
            elem.push_attribute(("volume", &*volume.unwrap()));
        }
        if rate.is_some() {
            elem.push_attribute(("rate", &*format!("{}", rate.unwrap())));
        }
        if pitch.is_some() {
            elem.push_attribute(("pitch", &*pitch.unwrap()));
        }
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <prosody> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_prosody_result = new_xml_writer.unwrap().end_ssml_prosody();
    /// assert!(end_prosody_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </prosody>
    /// ```
    pub fn end_ssml_prosody(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"prosody")))?)
    }

    /// Starts an SSML Sentence Tag. The Sentence Tag is useful for breaking
    /// up multiple sentences of text. AWS Polly follows the W3C SSML v1.1 Standard Here.
    /// As such the documentation for the sentence tag can be found:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_sentence).
    ///
    /// You can find the AWS Documentation that mentions the sentence tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#s-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_s_result = new_xml_writer.unwrap().start_ssml_sentence();
    /// assert!(start_s_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <s>
    /// ```
    pub fn start_ssml_sentence(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::Start(BytesStart::owned(b"s".to_vec(), "s".len())))?)
    }

    /// Ends an SSML <s> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_s_result = new_xml_writer.unwrap().end_ssml_sentence();
    /// assert!(end_s_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </s>
    /// ```
    pub fn end_ssml_sentence(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"s")))?)
    }

    /// Starts an SSML say-as Tag. The say-as tag is used for determing how a body of text
    /// should be interpreted, for example a phone number, or if you want something spelled
    /// out letter by letter. However AWS polly only supports the `interpret-as` attribute
    /// which is required, and does not support the `format`, and `detail` attributes.
    /// However for posterity you can read the W3C SSML v1.1 Spec:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_say-as).
    /// It should be noted the parameter for interpret-as is kept dynamic, since in the
    /// spec it says this list ***should*** change rapidly.
    ///
    /// You can find the AWS Documentation that mentions the say-as tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#say-as-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_say_as_result = new_xml_writer.unwrap().start_ssml_say_as("character".to_owned());
    /// assert!(start_say_as_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <say-as interpret-as="character">
    /// ```
    pub fn start_ssml_say_as(&mut self, interpret_as: String) -> Result<()> {
        let mut elem = BytesStart::owned(b"say-as".to_vec(), "say-as".len());
        elem.push_attribute(("interpret-as", &*interpret_as));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <say-as> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_say_as_result = new_xml_writer.unwrap().end_ssml_say_as();
    /// assert!(end_say_as_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </say-as>
    /// ```
    pub fn end_ssml_say_as(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"say-as")))?)
    }

    /// Starts an SSML sub Tag. The sub tag is used for a substitution of a word.
    /// For example in elememtal symbols you may want to show the elememtal symbol, but have
    /// the engine say the actual element name. AWS polly follows the W3C SSML v1.1 Spec:
    /// [HERE](https://www.w3.org/TR/2010/REC-speech-synthesis11-20100907/#edef_sub).
    ///
    /// You can find the AWS Documentation that mentions the sub tag:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#sub-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_sub_result = new_xml_writer.unwrap().start_ssml_sub("mercury".to_owned());
    /// assert!(start_sub_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <sub alias="mercury">
    /// ```
    pub fn start_ssml_sub(&mut self, alias: String) -> Result<()> {
        let mut elem = BytesStart::owned(b"sub".to_vec(), "sub".len());
        elem.push_attribute(("alias", &*alias));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <sub> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_sub_result = new_xml_writer.unwrap().end_ssml_sub();
    /// assert!(end_sub_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </sub>
    /// ```
    pub fn end_ssml_sub(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"sub")))?)
    }

    /// Starts an SSML Word/Token tag. The Word/Token tag for AWS Polly also deviates pretty
    /// far from the W3C Spec. So here like a few tags who shall not be named I will also
    /// only ilnk to the AWS Documentation for this tag. Which can be found:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html#w-tag).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::WordRole;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_w_result = new_xml_writer.unwrap().start_ssml_w(WordRole::Verb);
    /// assert!(start_w_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <w role="amazon:VB">
    /// ```
    pub fn start_ssml_w(&mut self, role: WordRole) -> Result<()> {
        let mut elem = BytesStart::owned(b"w".to_vec(), "w".len());
        elem.push_attribute(("role", &*format!("{}", role)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <w> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_w_result = new_xml_writer.unwrap().end_ssml_w();
    /// assert!(end_w_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </w>
    /// ```
    pub fn end_ssml_w(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"w")))?)
    }

    /// Starts an SSML amazon domain tag. These tags are unique to AWS Polly. As such
    /// the only place they are documented is inside the AWS Docs themsleves which are:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::AmazonDomainNames;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_effect_result = new_xml_writer.unwrap()
    ///   .start_ssml_amazon_domain(AmazonDomainNames::News);
    /// assert!(start_amazon_effect_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:domain name="news">
    /// ```
    pub fn start_ssml_amazon_domain(&mut self, name: AmazonDomainNames) -> Result<()> {
        let mut elem = BytesStart::owned(b"amazon:domain".to_vec(), "amazon:domain".len());
        elem.push_attribute(("name", &*format!("{}", name)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <amazon:domain> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_amazon_effect_result = new_xml_writer.unwrap().end_ssml_amazon_domain();
    /// assert!(end_amazon_effect_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </amazon:domain>
    /// ```
    pub fn end_ssml_amazon_domain(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"amazon:domain")))?)
    }

    /// Starts an SSML amazon effect tag. These tags are unique to AWS Polly. As such
    /// the only place they are documented is inside the AWS Docs themsleves which are:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::AmazonEffect;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_effect_result = new_xml_writer.unwrap()
    ///   .start_ssml_amazon_effect(AmazonEffect::Whispered);
    /// assert!(start_amazon_effect_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:effect name="whispered">
    /// ```
    pub fn start_ssml_amazon_effect(&mut self, name: AmazonEffect) -> Result<()> {
        let mut elem = BytesStart::owned(b"amazon:effect".to_vec(), "amazon:effect".len());
        elem.push_attribute(("name", &*format!("{}", name)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <amazon:effect> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_amazon_effect_result = new_xml_writer.unwrap().end_ssml_amazon_effect();
    /// assert!(end_amazon_effect_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </amazon:effect>
    /// ```
    pub fn end_ssml_amazon_effect(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"amazon:effect")))?)
    }

    /// Starts an SSML vocal tract tag. These tags are unique to AWS Polly. As such
    /// the only place they are documented is inside the AWS Docs themsleves which are:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_effect_result = new_xml_writer.unwrap()
    ///   .start_ssml_vocal_tract_length("+10%".to_owned());
    /// assert!(start_amazon_effect_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:effect vocal-tract-length="+10%">
    /// ```
    pub fn start_ssml_vocal_tract_length(&mut self, factor: String) -> Result<()> {
        let mut elem = BytesStart::owned(b"amazon:effect".to_vec(), "amazon:effect".len());
        elem.push_attribute(("vocal-tract-length", &*format!("{}", factor)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Starts an SSML phonation tag. These tags are unique to AWS Polly. As such
    /// the only place they are documented is inside the AWS Docs themsleves which are:
    /// [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html).
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::PhonationVolume;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_phonation_result = new_xml_writer.unwrap()
    ///   .start_ssml_phonation(PhonationVolume::Soft);
    /// assert!(start_amazon_phonation_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:effect phonation="soft">
    /// ```
    pub fn start_ssml_phonation(&mut self, volume: PhonationVolume) -> Result<()> {
        let mut elem = BytesStart::owned(b"amazon:effect".to_vec(), "amazon:effect".len());
        elem.push_attribute(("phonation", &*format!("{}", volume)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Starts an SSML <amazon:auto-breaths> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::*;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_auto_breaths_result = new_xml_writer.unwrap().start_ssml_auto_breaths(
    ///   BreathVolumes::Def,
    ///   AutoBreathFrequency::Def,
    ///   BreathDuration::Def,
    /// );
    /// assert!(start_amazon_auto_breaths_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:auto-breaths volume="default" frequency="default" duration="default">
    /// ```
    pub fn start_ssml_auto_breaths(
        &mut self,
        volume: BreathVolumes,
        frequency: AutoBreathFrequency,
        duration: BreathDuration,
    ) -> Result<()> {
        let mut elem =
            BytesStart::owned(b"amazon:auto-breaths".to_vec(), "amazon:auto-breaths".len());
        elem.push_attribute(("volume", &*format!("{}", volume)));
        elem.push_attribute(("frequency", &*format!("{}", frequency)));
        elem.push_attribute(("duration", &*format!("{}", duration)));
        Ok(self.writer.write_event(Event::Start(elem))?)
    }

    /// Ends an SSML <amazon:auto-breaths> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let end_amazon_auto_breaths_result = new_xml_writer.unwrap().end_ssml_amazon_auto_breaths();
    /// assert!(end_amazon_auto_breaths_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// </amazon:auto-breaths>
    /// ```
    pub fn end_ssml_amazon_auto_breaths(&mut self) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::End(BytesEnd::borrowed(b"amazon:auto-breaths")))?)
    }

    /// Starts an SSML <amazon:breath> tag.
    ///
    /// # Examples
    ///
    /// Rust Code:
    ///
    /// ```rust
    /// use text_to_polly_ssml::xml_writer::XmlWriter;
    /// use text_to_polly_ssml::ssml_constants::*;
    /// let mut new_xml_writer = XmlWriter::new();
    /// assert!(new_xml_writer.is_ok());
    /// let start_amazon_breath_result = new_xml_writer.unwrap().write_amazon_breath(
    ///   BreathVolumes::Def,
    ///   BreathDuration::Def,
    /// );
    /// assert!(start_amazon_breath_result.is_ok());
    /// ```
    ///
    /// Generated SSML:
    ///
    /// ```text
    /// <?xml version="1.0"?>
    /// <amazon:breath volume="default" duration="default" />
    /// ```
    pub fn write_amazon_breath(
        &mut self,
        volume: BreathVolumes,
        duration: BreathDuration,
    ) -> Result<()> {
        let mut elem = BytesStart::owned(b"amazon:breath".to_vec(), "amazon:breath".len());
        elem.push_attribute(("volume", &*format!("{}", volume)));
        elem.push_attribute(("duration", &*format!("{}", duration)));

        Ok(self.writer.write_event(Event::Empty(elem))?)
    }

    /// Writes some raw text to the XML Document. Should only be used inbetween <p> tags.
    pub fn write_text(&mut self, text: &str) -> Result<()> {
        Ok(self
            .writer
            .write_event(Event::Text(BytesText::from_plain_str(text)))?)
    }

    /// Renders the XML document in it's current state. This expects the document
    /// to be completely valid UTF-8, and will do no closing of tags for you.
    pub fn render(&mut self) -> String {
        String::from_utf8(self.writer.clone().into_inner().into_inner())
            .expect("SSML is not valid UTF-8!")
    }
}
