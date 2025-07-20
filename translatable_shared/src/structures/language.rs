use std::str::FromStr;
use std::fmt::{Display, Formatter, Result as FmtResult, Write};
use std::error::Error;

use edit_distance::edit_distance;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{format_ident, quote, ToTokens};
use strum::{EnumIter, EnumProperty, IntoEnumIterator};
use syn::parse::{Parse, ParseStream};
use syn::{Result as SynResult, Error as SynError};

use crate::utils::option_stream;

#[derive(Debug, Clone)]
pub struct LanguageError {
    attempt: String,
    closest: Option<String>
}

impl LanguageError {
    pub fn from_data(attempt: String, closest: Option<String>) -> Self {
        Self {
            attempt,
            closest
        }
    }

    #[inline(always)]
    #[cold]
    pub fn attempt(&self) -> &str {
        &self.attempt
    }

    #[inline(always)]
    #[cold]
    pub const fn closest(&self) -> Option<&String> {
        self.closest.as_ref()
    }
}

impl ToTokens for LanguageError {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let attempt = &self.attempt;
        let closest = option_stream(&self.closest);

        tokens.extend(
            quote! {
                ::translatable::prelude::LanguageError::from_data(
                    #attempt.to_string(),
                    #closest
                )
            }
        )
    }
}

impl Display for LanguageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, r#""{}" is not a valid language"#, self.attempt)?;

        match &self.closest {
            Some(closest) => write!(f, r#", perhaps you meant "{closest}"?"#),
            None => write!(f, ".")
        }
    }
}

impl Error for LanguageError {}

impl Language {
    fn possible_idents(&self) -> Vec<String> {
        let mut result = self.get_str("Alternatives")
            .map(|alts| alts
                .split(",")
                .map(|a| a.trim().to_string())
                .collect()
            )
            .unwrap_or_else(Vec::new);

        if let Some(name) = self.get_str("Name") {
            result.push(name.to_string());
        }

        result.push(format!("{self:?}"));

        result
    }

    #[cold]
    fn close_to(attempt: &str) -> Option<String> {
        Self::iter()
            .flat_map(|curr| curr.possible_idents())
            .min_by_key(|candidate| edit_distance(attempt, &candidate))
    }
}

impl Parse for Language {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut raw = String::new();
        let mut spans = Vec::new();

        while !input.is_empty() {
            match input.parse::<TokenTree>()? {
                TokenTree::Ident(ident) => {
                    let _ = write!(raw, "{ident:#}");
                    spans.push(ident.span());
                },

                TokenTree::Punct(punct) => match punct.as_char() {
                    ',' | ';' => break,
                    other => {
                        let _ = write!(raw, "{other:#}");
                        spans.push(punct.span());
                    }
                },

                other => {
                    return Err(SynError::new_spanned(
                        other,
                        "Only sequences of identifiers and punctuations are allowed while parsing a Language."
                    ))
                }
            }
        }

        match raw.parse::<Language>() {
            Ok(language) => Ok(language),
            Err(err) => {
                let overall_span = {
                    let mut span_iter = spans.into_iter();

                    span_iter.next()
                        .map(|first| span_iter
                            .fold(first, |acc, span| acc.join(span).unwrap_or(acc))
                        )
                        .unwrap_or(Span::call_site())
                };

                Err(SynError::new(overall_span, err))
            }
        }
    }
}

impl FromStr for Language {
    type Err = LanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .find(|item| item
                .possible_idents()
                .iter()
                .any(|ident| ident == &s)
            )
            .ok_or_else(|| LanguageError {
                attempt: s.to_string(),
                closest: Language::close_to(s)
            })
    }
}

impl ToTokens for Language {
    #[inline(always)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut target_lang = String::with_capacity(2);
        let _ = write!(&mut target_lang, "{self:?}");

        let ident = format_ident!("{target_lang:?}");

        tokens.extend(
            quote! { ::translatable::prelude::Language::#ident }
        );
    }
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self.get_str("Name") {
            Some(name) => write!(f, "{name}"),
            None => write!(f, "{self:?}")
        }
    }
}

#[derive(EnumProperty, EnumIter, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Language {
    #[strum(props(Name = "Afar", Alternatives = "Afaraf"))]
    Aa,

    #[strum(props(Name = "Abkhaz", Alternatives = "аҧсшәа"))]
    Ab,

    #[strum(props(Name = "Avestan", Alternatives = "Avesta"))]
    Ae,

    #[strum(props(Name = "Afrikaans"))]
    Af,

    #[strum(props(Name = "Akan"))]
    Ak,

    #[strum(props(Name = "Amharic", Alternatives = "አማርኛ"))]
    Am,

    #[strum(props(Name = "Aragonese", Alternatives = "aragonés"))]
    An,

    #[strum(props(Name = "Arabic", Alternatives = "العربية"))]
    Ar,

    #[strum(props(Name = "Assamese", Alternatives = "অসমীয়া"))]
    As,

    #[strum(props(Name = "Avaric", Alternatives = "авар мацӀ, магӀарул мацӀ"))]
    Av,

    #[strum(props(Name = "Aymara", Alternatives = "aymar aru"))]
    Ay,

    #[strum(props(Name = "Azerbaijani", Alternatives = "azərbaycan dili"))]
    Az,

    #[strum(props(Name = "Bashkir", Alternatives = "башҡорт теле"))]
    Ba,

    #[strum(props(Name = "Belarusian", Alternatives = "беларуская мова"))]
    Be,

    #[strum(props(Name = "Bulgarian", Alternatives = "български език"))]
    Bg,

    #[strum(props(Name = "Bihari", Alternatives = "भोजपुरी"))]
    Bh,

    #[strum(props(Name = "Bislama"))]
    Bi,

    #[strum(props(Name = "Bambara", Alternatives = "bamanankan"))]
    Bm,

    #[strum(props(Name = "Bengali", Alternatives = "Bangla, বাংলা"))]
    Bn,

    #[strum(props(Name = "Tibetan", Alternatives = "Tibetan Standard, བོད་ཡིག"))]
    Bo,

    #[strum(props(Name = "Breton", Alternatives = "brezhoneg"))]
    Br,

    #[strum(props(Name = "Bosnian", Alternatives = "bosanski jezik"))]
    Bs,

    #[strum(props(Name = "Catalan", Alternatives = "català"))]
    Ca,

    #[strum(props(Name = "Chechen", Alternatives = "нохчийн мотт"))]
    Ce,

    #[strum(props(Name = "Chamorro", Alternatives = "Chamoru"))]
    Ch,

    #[strum(props(Name = "Corsican", Alternatives = "corsu, lingua corsa"))]
    Co,

    #[strum(props(Name = "Cree", Alternatives = "ᓀᐦᐃᔭᐍᐏᐣ"))]
    Cr,

    #[strum(props(Name = "Czech", Alternatives = "čeština, český jazyk"))]
    Cs,

    #[strum(props(Name = "Church Slavonic", Alternatives = "Old Church Slavonic, Old Bulgarian, ѩзыкъ словѣньскъ"))]
    Cu,

    #[strum(props(Name = "Chuvash", Alternatives = "чӑваш чӗлхи"))]
    Cv,

    #[strum(props(Name = "Welsh", Alternatives = "Cymraeg"))]
    Cy,

    #[strum(props(Name = "Danish", Alternatives = "dansk"))]
    Da,

    #[strum(props(Name = "German", Alternatives = "Deutsch"))]
    De,

    #[strum(props(Name = "Divehi", Alternatives = "Dhivehi, Maldivian, ދިވެހި"))]
    Dv,

    #[strum(props(Name = "Dzongkha", Alternatives = "རྫོང་ཁ"))]
    Dz,

    #[strum(props(Name = "Ewe", Alternatives = "Eʋegbe"))]
    Ee,

    #[strum(props(Name = "Greek", Alternatives = "modern, ελληνικά"))]
    El,

    #[strum(props(Name = "English"))]
    En,

    #[strum(props(Name = "Esperanto"))]
    Eo,

    #[strum(props(Name = "Spanish", Alternatives = "Español"))]
    Es,

    #[strum(props(Name = "Estonian", Alternatives = "eesti, eesti keel"))]
    Et,

    #[strum(props(Name = "Basque", Alternatives = "euskara, euskera"))]
    Eu,

    #[strum(props(Name = "Persian", Alternatives = "Farsi, فارسی"))]
    Fa,

    #[strum(props(Name = "Fula", Alternatives = "Fulah, Pulaar, Pular, Fulfulde"))]
    Ff,

    #[strum(props(Name = "Finnish", Alternatives = "suomi, suomen kieli"))]
    Fi,

    #[strum(props(Name = "Fijian", Alternatives = "vosa Vakaviti"))]
    Fj,

    #[strum(props(Name = "Faroese", Alternatives = "føroyskt"))]
    Fo,

    #[strum(props(Name = "French", Alternatives = "français"))]
    Fr,

    #[strum(props(Name = "Western Frisian", Alternatives = "Frysk"))]
    Fy,

    #[strum(props(Name = "Irish", Alternatives = "Gaeilge"))]
    Ga,

    #[strum(props(Name = "Scottish Gaelic", Alternatives = "Gaelic, Gàidhlig"))]
    Gd,

    #[strum(props(Name = "Galician", Alternatives = "galego"))]
    Gl,

    #[strum(props(Name = "Guaraní", Alternatives = "Avañe'ẽ"))]
    Gn,

    #[strum(props(Name = "Gujarati", Alternatives = "ગુજરાતી"))]
    Gu,

    #[strum(props(Name = "Manx", Alternatives = "Gaelg, Gailck"))]
    Gv,

    #[strum(props(Name = "Hausa", Alternatives = "Hausa, هَوُسَ"))]
    Ha,

    #[strum(props(Name = "Hebrew", Alternatives = "עברית"))]
    He,

    #[strum(props(Name = "Hindi", Alternatives = "हिन्दी, हिंदी"))]
    Hi,

    #[strum(props(Name = "Hiri Motu"))]
    Ho,

    #[strum(props(Name = "Croatian", Alternatives = "hrvatski jezik"))]
    Hr,

    #[strum(props(Name = "Haitian", Alternatives = "Haitian Creole, Kreyòl ayisyen"))]
    Ht,

    #[strum(props(Name = "Hungarian", Alternatives = "magyar"))]
    Hu,

    #[strum(props(Name = "Armenian", Alternatives = "Հայերեն"))]
    Hy,

    #[strum(props(Name = "Herero", Alternatives = "Otjiherero"))]
    Hz,

    #[strum(props(Name = "Interlingua"))]
    Ia,

    #[strum(props(Name = "Indonesian", Alternatives = "Bahasa Indonesia"))]
    Id,

    #[strum(props(Name = "Interlingue", Alternatives = "Occidental"))]
    Ie,

    #[strum(props(Name = "Igbo", Alternatives = "Asụsụ Igbo"))]
    Ig,

    #[strum(props(Name = "Nuosu", Alternatives = "ꆈꌠ꒿ Nuosuhxop"))]
    Ii,

    #[strum(props(Name = "Inupiaq", Alternatives = "Iñupiaq, Iñupiatun"))]
    Ik,

    #[strum(props(Name = "Ido"))]
    Io,

    #[strum(props(Name = "Icelandic", Alternatives = "Íslenska"))]
    Is,

    #[strum(props(Name = "Italian", Alternatives = "Italiano"))]
    It,

    #[strum(props(Name = "Inuktitut", Alternatives = "ᐃᓄᒃᑎᑐᑦ"))]
    Iu,

    #[strum(props(Name = "Japanese", Alternatives = "日本語, にほんご"))]
    Ja,

    #[strum(props(Name = "Javanese", Alternatives = "ꦧꦱꦗꦮ, Basa Jawa"))]
    Jv,

    #[strum(props(Name = "Georgian", Alternatives = "ქართული"))]
    Ka,

    #[strum(props(Name = "Kongo", Alternatives = "Kikongo"))]
    Kg,

    #[strum(props(Name = "Kikuyu", Alternatives = "Gikuyu, Gĩkũyũ"))]
    Ki,

    #[strum(props(Name = "Kwanyama", Alternatives = "Kuanyama, Kuanyama"))]
    Kj,

    #[strum(props(Name = "Kazakh", Alternatives = "қазақ тілі"))]
    Kk,

    #[strum(props(Name = "Kalaallisut", Alternatives = "Greenlandic, kalaallisut, kalaallit oqaasii"))]
    Kl,

    #[strum(props(Name = "Khmer", Alternatives = "ខ្មែរ, ខេមរភាសា, ភាសាខ្មែរ"))]
    Km,

    #[strum(props(Name = "Kannada", Alternatives = "ಕನ್ನಡ"))]
    Kn,

    #[strum(props(Name = "Korean", Alternatives = "한국어"))]
    Ko,

    #[strum(props(Name = "Kanuri"))]
    Kr,

    #[strum(props(Name = "Kashmiri", Alternatives = "कश्मीरी, كشميري"))]
    Ks,

    #[strum(props(Name = "Kurdish", Alternatives = "Kurdî, كوردی"))]
    Ku,

    #[strum(props(Name = "Komi", Alternatives = "коми кыв"))]
    Kv,

    #[strum(props(Name = "Cornish", Alternatives = "Kernewek"))]
    Kw,

    #[strum(props(Name = "Kyrgyz", Alternatives = "Кыргызча, Кыргыз тили"))]
    Ky,

    #[strum(props(Name = "Latin", Alternatives = "latine, lingua latina"))]
    La,

    #[strum(props(Name = "Luxembourgish", Alternatives = "Letzeburgesch, Lëtzebuergesch"))]
    Lb,

    #[strum(props(Name = "Ganda", Alternatives = "Luganda"))]
    Lg,

    #[strum(props(Name = "Limburgish", Alternatives = "Limburgan, Limburger, Limburgs"))]
    Li,

    #[strum(props(Name = "Lingala", Alternatives = "Lingála"))]
    Ln,

    #[strum(props(Name = "Lao", Alternatives = "ພາສາລາວ"))]
    Lo,

    #[strum(props(Name = "Lithuanian", Alternatives = "lietuvių kalba"))]
    Lt,

    #[strum(props(Name = "Luba-Katanga", Alternatives = "Tshiluba"))]
    Lu,

    #[strum(props(Name = "Latvian", Alternatives = "latviešu valoda"))]
    Lv,

    #[strum(props(Name = "Malagasy", Alternatives = "fiteny malagasy"))]
    Mg,

    #[strum(props(Name = "Marshallese", Alternatives = "Kajin M̧ajeļ"))]
    Mh,

    #[strum(props(Name = "Māori", Alternatives = "te reo Māori"))]
    Mi,

    #[strum(props(Name = "Macedonian", Alternatives = "македонски јазик"))]
    Mk,

    #[strum(props(Name = "Malayalam", Alternatives = "മലയാളം"))]
    Ml,

    #[strum(props(Name = "Mongolian", Alternatives = "Монгол хэл"))]
    Mn,

    #[strum(props(Name = "Marathi", Alternatives = "Marāṭhī, मराठी"))]
    Mr,

    #[strum(props(Name = "Malay", Alternatives = "bahasa Melayu, بهاس ملايو"))]
    Ms,

    #[strum(props(Name = "Maltese", Alternatives = "Malti"))]
    Mt,

    #[strum(props(Name = "Burmese", Alternatives = "ဗမာစာ"))]
    My,

    #[strum(props(Name = "Nauruan", Alternatives = "Dorerin Naoero"))]
    Na,

    #[strum(props(Name = "Norwegian Bokmål", Alternatives = "Norsk bokmål"))]
    Nb,

    #[strum(props(Name = "Northen Ndbele", Alternatives = "isiNdebele"))]
    Nd,

    #[strum(props(Name = "Nepali", Alternatives = "नेपाली"))]
    Ne,

    #[strum(props(Name = "Ndonga", Alternatives = "Owambo"))]
    Ng,

    #[strum(props(Name = "Dutch", Alternatives = "Nederlands, Vlaams"))]
    Nl,

    #[strum(props(Name = "Norwegian Nynorsk", Alternatives = "Norsk nynorsk"))]
    Nn,

    #[strum(props(Name = "Norwegian", Alternatives = "Norsk"))]
    No,

    #[strum(props(Name = "Southen Ndebele", Alternatives = "isiNdebele"))]
    Nr,

    #[strum(props(Name = "Navajo", Alternatives = "Navaho, Diné bizaad"))]
    Nv,

    #[strum(props(Name = "Chichewa", Alternatives = "Chewa, Nyanja, chiCheŵa, chinyanja"))]
    Ny,

    #[strum(props(Name = "Occitan", Alternatives = "occitan, lenga d'òc"))]
    Oc,

    #[strum(props(Name = "Ojibwe", Alternatives = "Ojibwa, ᐊᓂᔑᓈᐯᒧᐎᓐ"))]
    Oj,

    #[strum(props(Name = "Oromo", Alternatives = "Afaan Oromoo"))]
    Om,

    #[strum(props(Name = "Oriya", Alternatives = "ଓଡ଼ିଆ"))]
    Or,

    #[strum(props(Name = "ossetian", Alternatives = "Ossetic, ирон æвзаг"))]
    Os,

    #[strum(props(Name = "Eastern Punjabi", Alternatives = "ਪੰਜਾਬੀ"))]
    Pa,

    #[strum(props(Name = "Pali", Alternatives = "Pāli, पाऴि"))]
    Pi,

    #[strum(props(Name = "Polish", Alternatives = "język polski, polszczyzna"))]
    Pl,

    #[strum(props(Name = "Pashto", Alternatives = "Pushto, پښتو"))]
    Ps,

    #[strum(props(Name = "Portuguese", Alternatives = "Português"))]
    Pt,

    #[strum(props(Name = "Quechua", Alternatives = "Runa Simi, Kichwa"))]
    Qu,

    #[strum(props(Name = "Romansh", Alternatives = "rumantsch grischun"))]
    Rm,

    #[strum(props(Name = "Kirundi", Alternatives = "Ikirundi"))]
    Rn,

    #[strum(props(Name = "Romanian", Alternatives = "Română"))]
    Ro,

    #[strum(props(Name = "Russian", Alternatives = "Русский"))]
    Ru,

    #[strum(props(Name = "Kinyarwanda", Alternatives = "Ikinyarwanda"))]
    Rw,

    #[strum(props(Name = "Sanskrit", Alternatives = "Saṁskṛta, संस्कृतम्"))]
    Sa,

    #[strum(props(Name = "Sardinian", Alternatives = "sardu"))]
    Sc,

    #[strum(props(Name = "Sindhi", Alternatives = "सिन्धी, سنڌي، سندھی"))]
    Sd,

    #[strum(props(Name = "Northern Sami", Alternatives = "Davvisámegiella"))]
    Se,

    #[strum(props(Name = "Sango", Alternatives = "yângâ tî sängö"))]
    Sg,

    #[strum(props(Name = "Sinhalese", Alternatives = "Sinhala, සිංහල"))]
    Si,

    #[strum(props(Name = "Slovak", Alternatives = "slovenčina, slovenský jazyk"))]
    Sk,

    #[strum(props(Name = "Slovene", Alternatives = "slovenski jezik, slovenščina"))]
    Sl,

    #[strum(props(Name = "Samoan", Alternatives = "gagana fa'a Samoa"))]
    Sm,

    #[strum(props(Name = "Shona", Alternatives = "chiShona"))]
    Sn,

    #[strum(props(Name = "Somali", Alternatives = "Soomaaliga, af Soomaali"))]
    So,

    #[strum(props(Name = "Albanian", Alternatives = "Shqip"))]
    Sq,

    #[strum(props(Name = "Serbian", Alternatives = "српски језик"))]
    Sr,

    #[strum(props(Name = "Swati", Alternatives = "SiSwati"))]
    Ss,

    #[strum(props(Name = "Southern Sotho", Alternatives = "Sesotho"))]
    St,

    #[strum(props(Name = "Sundanese", Alternatives = "Basa Sunda"))]
    Su,

    #[strum(props(Name = "Swedish", Alternatives = "svenska"))]
    Sv,

    #[strum(props(Name = "Swahili", Alternatives = "Kiswahili"))]
    Sw,

    #[strum(props(Name = "Tamil", Alternatives = "தமிழ்"))]
    Ta,

    #[strum(props(Name = "Telugu", Alternatives = "తెలుగు"))]
    Te,

    #[strum(props(Name = "Tajik", Alternatives = "тоҷикӣ, toçikī, تاجیکی"))]
    Tg,

    #[strum(props(Name = "Thai", Alternatives = "ไทย"))]
    Th,

    #[strum(props(Name = "Tigrinya", Alternatives = "ትግርኛ"))]
    Ti,

    #[strum(props(Name = "Turkmen", Alternatives = "Türkmen, Түркмен"))]
    Tk,

    #[strum(props(Name = "Tagalog", Alternatives = "Wikang Tagalog"))]
    Tl,

    #[strum(props(Name = "Tswana", Alternatives = "Setswana"))]
    Tn,

    #[strum(props(Name = "Tonga", Alternatives = "Tonga Islands, faka Tonga"))]
    To,

    #[strum(props(Name = "Turkish", Alternatives = "Türkçe"))]
    Tr,

    #[strum(props(Name = "Tsonga", Alternatives = "Xitsonga"))]
    Ts,

    #[strum(props(Name = "Tatar", Alternatives = "татар теле, tatar tele"))]
    Tt,

    #[strum(props(Name = "Twi"))]
    Tw,

    #[strum(props(Name = "Tahitian", Alternatives = "Reo Tahiti"))]
    Ty,

    #[strum(props(Name = "Uyghur", Alternatives = "ئۇيغۇرچە, Uyghurche"))]
    Ug,

    #[strum(props(Name = "Ukrainian", Alternatives = "Українська"))]
    Uk,

    #[strum(props(Name = "Urdu", Alternatives = "اردو"))]
    Ur,

    #[strum(props(Name = "Uzbek", Alternatives = "Oʻzbek, Ўзбек, أۇزبېك"))]
    Uz,

    #[strum(props(Name = "Venda", Alternatives = "Tshivenḓa"))]
    Ve,

    #[strum(props(Name = "Vietnamese", Alternatives = "Tiếng Việt"))]
    Vi,

    #[strum(props(Name = "Volapük"))]
    Vo,

    #[strum(props(Name = "Walloon", Alternatives = "walon"))]
    Wa,

    #[strum(props(Name = "Wolof", Alternatives = "Wollof"))]
    Wo,

    #[strum(props(Name = "Xhosa", Alternatives = "isiXhosa"))]
    Xh,

    #[strum(props(Name = "Yiddish", Alternatives = "ייִדיש"))]
    Yi,

    #[strum(props(Name = "Yoruba", Alternatives = "Yorùbá"))]
    Yo,

    #[strum(props(Name = "Zhuang", Alternatives = "Chuang, Saɯ cueŋƅ, Saw cuengh"))]
    Za,

    #[strum(props(Name = "Chinese", Alternatives = "中文, 汉语, 漢語"))]
    Zh,

    #[strum(props(Name = "Zulu", Alternatives = "isiZulu"))]
    Zu,
}
