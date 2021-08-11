use combine::{
    Parser, Stream,
    many1, error::{ParseError},
    parser::char::{char, spaces, letter, digit},
    optional, 
};

#[derive(Debug, PartialEq)]
pub struct Test {
    pub library: String,
    pub parameters: Vec<Param>,
}


#[derive(Debug, PartialEq)]
pub struct Param {
    pub arg_direction: ArgDirection,
    pub param_type: ParamType,
    pub name: String, 
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub enum ParamType {
    Sigrefexpr,
    Voltage, 
    Current,
    String,
    Integer,
    Real,
    Unknown,
}

#[derive(Debug, PartialEq)]
pub enum ArgDirection {
    In,
    Out,
    Unknown,
}

fn digits_and_unit<Input>() -> impl Parser<Input, Output = (f32, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        optional(char('-')).skip(spaces()),
        many1::<String, _, _>(digit()),
        optional(char('.').with(many1::<String, _, _>(digit()))),
        many1::<String, _, _>(letter()),
    )
        .map(|(sign, int, dec, unit)| {
            let mut flt: String = String::new();
            match sign {
                Some('-') => flt.push('-'),
                Some(_) => {},
                None => {},
            };
            flt.push_str(&int);
            match dec {
                Some(ref a) => {
                    flt.push('.');
                    flt.push_str(&a);
                }
                None => {}
            }
            let result: f32 = flt.parse::<f32>().unwrap_or(f32::MIN);
            (result, unit)
        })
}

fn parameter<Input>() -> impl Parser<Input, Output = (String, String, String, f32, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
//    (
//        many1(letter()).skip(spaces()),
//        many1(letter()).skip(spaces()),
//        many1(letter()).skip(spaces()),
//        char('=').skip(spaces()),
//        many1(letter()).skip(spaces()),
//        char(';').skip(spaces()),
//    ).map(|(arg, param, name, _, value, _)| Param {
//        arg_direction: match &arg {
//            "In" => ArgDirection::In,
//            "Out" => ArgDirection::Out,
//            _ => ArgDirection::Unknown,
//        },
//        param_type: match &param {
//            "sigref_expr" => ParamType::Sigrefexpr,
//            "Voltage" => ParamType::Voltage,
//            "Current" => ParamType::Current,
//            "String" => ParamType::String,
//            "Integer" => ParamType::Integer,
//            "Real" => ParamType::Real,
//            _ => ParamType::Unknown,
//        },
//        name: name,
//        value: value,
//    })
    (
        many1::<String, _, _>(letter()).skip(spaces()),
        many1::<String, _, _>(letter()).skip(spaces()),
        many1::<String, _, _>(letter()).skip(spaces()),
        char('=').skip(spaces()),
        digits_and_unit().skip(spaces()),
        char(';').skip(spaces()),
    )
        .map(|(arg, param, name, _, (digit, unit), _)| (arg, param, name, digit, unit))
        .message("hogehoge")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Parsing test of parameter
    #[test]
    fn test_parse_parameter() {
        let result = parameter().parse("In Voltage Lower = -1.3V;");
        assert_eq!(result, Ok((("In".to_string(), "Voltage".to_string(), "Lower".to_string(), -1.3, "V".to_string()), "")));
    }

    #[test]
    fn test_digit_and_unit() {
        let result1 = digits_and_unit().parse("12345V");
        let result2 = digits_and_unit().parse("-12.345A");
        let result3 = digits_and_unit().parse("A");

        assert_eq!(result1, Ok(((12345f32, "V".to_string()), "")));
        assert_eq!(result2, Ok(((-12.345, "A".to_string()), "")));
        assert!(result3.is_err());
    }
}
