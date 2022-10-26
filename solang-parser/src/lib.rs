// SPDX-License-Identifier: Apache-2.0

//! Solidity file parser
use crate::pt::CodeLocation;
use diagnostics::Diagnostic;
use lalrpop_util::ParseError;


pub mod diagnostics;
pub mod doccomment;
pub mod lexer;
pub mod pt;
#[cfg(test)]
mod test;
#[allow(clippy::all)]
mod solidity {
    include!(concat!(env!("OUT_DIR"), "/solidity.rs"));
}

/// Parse solidity file
pub fn parse(
    src: &str,
    file_no: usize,
) -> Result<(pt::SourceUnit, Vec<pt::Comment>), Vec<Diagnostic>> {
    // parse phase
    let mut comments = Vec::new();

    let lex = lexer::Lexer::new(src, file_no, &mut comments);
    let my_errors= &mut Vec::new();
    println!("SRCCCC {:?}", src);
    let s = solidity::SourceUnitParser::new().parse(src, file_no, my_errors, lex);
    println!("S {:?}",s);
    /*if let Err(errr_debug) = s.clone() {
        //println!("S {:?}", errr_debug);
    }*/
    
    //println!("my_errs {:?}", my_errors);
    /*let  errors =&mut Vec::new();
    if !my_errors.is_empty(){
    for iter in my_errors {
        //print!("iter {:?}", iter);
       match &iter.error {
            ParseError::InvalidToken { location } => errors.push(Diagnostic::parser_error(
                pt::Loc::File(file_no, *location, *location),
                "invalid token".to_string(),
            )),
            ParseError::UnrecognizedToken {
                token: (l, token, r),
                expected,
            } => errors.push(Diagnostic::parser_error(
                pt::Loc::File(file_no, *l, *r),
                format!(
                    "unrecognised token '{}', expected {}",
                    token,
                    expected.join(", ")
                ),
            )),
            ParseError::User { error } => errors.push(Diagnostic::parser_error(error.loc(), error.to_string())),
            ParseError::ExtraToken { token } => errors.push(Diagnostic::parser_error(
                pt::Loc::File(file_no, token.0, token.2),
                format!("extra token '{}' encountered", token.0),
            )),
            ParseError::UnrecognizedEOF { location, expected } => errors.push(Diagnostic::parser_error(
                pt::Loc::File(file_no, *location, *location),
                format!("unexpected end of file, expecting {}", expected.join(", ")),
            )),
        };

        
    }
    return Err(errors.to_vec());
}*/


    if let Err(e) = s {
        let errors = vec![match e {
            ParseError::InvalidToken { location } => Diagnostic::parser_error(
                pt::Loc::File(file_no, location, location),
                "invalid token".to_string(),
            ),
            ParseError::UnrecognizedToken {
                token: (l, token, r),
                expected,
            } => Diagnostic::parser_error(
                pt::Loc::File(file_no, l, r),
                format!(
                    "unrecognised token '{}', expected {}",
                    token,
                    expected.join(", ")
                ),
            ),
            ParseError::User { error } => Diagnostic::parser_error(error.loc(), error.to_string()),
            ParseError::ExtraToken { token } => Diagnostic::parser_error(
                pt::Loc::File(file_no, token.0, token.2),
                format!("extra token '{}' encountered", token.0),
            ),
            ParseError::UnrecognizedEOF { location, expected } => Diagnostic::parser_error(
                pt::Loc::File(file_no, location, location),
                format!("unexpected end of file, expecting {}", expected.join(", ")),
            ),
        }];

        Err(errors)
    } else {
        Ok((s.unwrap(), comments))
    }
}
