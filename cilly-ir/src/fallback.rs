use std::{collections::HashMap, sync::Mutex};

use qparse::Parseable;

use crate::{Fnc, ModuleBuilderError};
#[qparse_macros::qparse("# FALLBACK ")]
struct LineDef;
#[derive(Debug)]
struct Fallback {
    src: String,
    instantiated: Mutex<HashMap<Vec<(String, String)>, Fnc>>,
}
impl Fallback {
    fn new(src: String) -> Self {
        Self {
            instantiated: Mutex::new(HashMap::default()),
            src,
        }
    }
    fn instantiate(&self, instance: Vec<(String, String)>) -> Fnc {
        self.instantiated
            .lock()
            .unwrap()
            .entry(instance.clone())
            .or_insert_with(|| {
                let mut src = self.src.clone();
                for (template, value) in instance {
                    src = src.replace(&template, &value);
                }
                // TODO: replace with parse with error handling.
                let res = Fnc::parse::<'_, nom_language::error::VerboseError<&str>>(&src);
                match res {
                    Ok((_, fnc)) => fnc,
                    Err(err) => {
                        let msg = match err {
                            nom::Err::Error(e) | nom::Err::Failure(e) => {
                                nom_language::error::convert_error(src.as_str(), e)
                            }
                            nom::Err::Incomplete(n) => panic!("incomplete: {n:?}"),
                        };
                        panic!("{msg}\nsrc:\n{src}")
                    }
                }
            })
            .clone()
    }
}
#[derive(Debug)]
struct FallbackList {
    fallbacks: HashMap<String, Fallback>,
}
impl FallbackList {
    fn new(fallback_str: &str) -> Self {
        let mut curr = String::new();
        let mut fallbacks = HashMap::default();
        let mut prev_name: Option<String> = None;
        for line in fallback_str.lines() {
            let Ok((reminder, _)) = LineDef::simple_parse(line) else {
                curr.push_str(line);
                curr.push('\n');
                continue;
            };
            let name = reminder.trim().to_string();
            if let Some(name) = prev_name {
                fallbacks.insert(name.clone(), Fallback::new(curr));
                curr = String::new();
            }
            prev_name = Some(name);
        }
        if let Some(name) = prev_name {
            fallbacks.insert(name.clone(), Fallback::new(curr));
        }
        Self { fallbacks }
    }
    fn instantiate<S: Into<String>, V: Into<Vec<(S, S)>>>(
        &self,
        name: &str,
        instance: V,
    ) -> Result<Fnc, ModuleBuilderError> {
        let v: Vec<(S, S)> = instance.into();
        let v: Vec<(String, String)> = v.into_iter().map(|(a, b)| (a.into(), b.into())).collect();
        Ok(self
            .fallbacks
            .get(name)
            .ok_or(ModuleBuilderError::FallbackNotFound {
                name: name.to_string(),
            })?
            .instantiate(v))
    }
}
#[test]
fn parse_add_emu() {
    //; CILLY_FALLBACK iadd $WIDE $NARROW
    let emu = include_str!("fallback/emu_iadd.cir");

    let list = FallbackList::new(&emu);
    let emu = list
        .instantiate(
            "cilly.emu.iadd",
            vec![("$NARROW", "64"), ("$WIDE", "128"), ("$IS_LE", "true")],
        )
        .unwrap();
    todo!("emu:{emu}");
}
