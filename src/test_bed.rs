use core::fmt;

use crate::http::Response;

#[derive(Default, Debug, Clone, Copy)]
pub enum Operator {
    #[default]
    Unknown,
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    GreaterThan,
    SmallerThan,
    GreaterThanOrEqualTo,
    SmallerThanOrEqualTo,
}

impl Operator {
    pub fn as_symbol(&self) -> &str {
        match self {
            Self::Unknown => "--",
            Self::Equals => "=",
            Self::NotEquals => "!=",
            Self::GreaterThan => ">",
            Self::GreaterThanOrEqualTo => ">=",
            Self::SmallerThan => "<",
            Self::SmallerThanOrEqualTo => "<=",
            Self::Contains => "~",
            Self::StartsWith => "^",
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_symbol())
    }
}

#[derive(Default, Debug)]
pub struct TestCase {
    pub key: String,
    pub value: String,
    pub operation: Operator,
}

pub fn test_case(response: &Response, case: &TestCase) -> bool {
    print!(
        "Case: {} {} {}",
        case.key,
        get_operator(case.operation.as_symbol()).unwrap_or(Operator::Unknown),
        case.value
    );
    if case.key == "body" {
        return do_operation(&case.operation, &response.body, &case.value);
    } else if case.key == "status" {
        return do_operation(&case.operation, &response.status.to_string(), &case.value);
    }

    let header_val = response.get_header(&case.key).unwrap_or("");
    do_operation(&case.operation, header_val, &case.value)
}

pub fn get_operator(op_str: &str) -> Result<Operator, String> {
    match op_str {
        "=" => Ok(Operator::Equals),
        "!=" => Ok(Operator::NotEquals),
        ">" => Ok(Operator::GreaterThan),
        ">=" => Ok(Operator::GreaterThanOrEqualTo),
        "<" => Ok(Operator::SmallerThan),
        "<=" => Ok(Operator::SmallerThanOrEqualTo),
        "~" => Ok(Operator::Contains),
        "^" => Ok(Operator::StartsWith),
        invalid => Err(format!("Syntax error, unknown symbol, {}", invalid)),
    }
}

pub fn do_operation(operation: &Operator, left: &str, right: &str) -> bool {
    match operation {
        Operator::Unknown => false,
        Operator::Equals => left == right,
        Operator::NotEquals => left != right,
        Operator::Contains => left.contains(right),
        Operator::StartsWith => left.starts_with(right),
        op @ (Operator::GreaterThan
        | Operator::SmallerThan
        | Operator::GreaterThanOrEqualTo
        | Operator::SmallerThanOrEqualTo) => {
            let l_val = left.parse::<f64>();
            let r_val = right.parse::<f64>();

            if let (Ok(l), Ok(r)) = (l_val, r_val) {
                match op {
                    Operator::GreaterThan => l > r,
                    Operator::SmallerThan => l < r,
                    Operator::GreaterThanOrEqualTo => l >= r,
                    Operator::SmallerThanOrEqualTo => l <= r,
                    _ => false,
                }
            } else {
                false
            }
        }
    }
}
