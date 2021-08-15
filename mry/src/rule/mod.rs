mod behavior;
mod matcher;

pub use behavior::*;
pub use matcher::*;

pub(crate) struct Rule<I, O> {
    pub matcher: Matcher<I>,
    pub behavior: Behavior<I, O>,
}

impl<I: PartialEq + Clone, O> Rule<I, O> {
    pub fn called(&mut self, input: &I) -> Option<O> {
        if self.matcher.matches(input) {
            return Some(self.behavior.called(input));
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn called_returns_none() {
        let mut rule: Rule<u8, u8> = Rule {
            matcher: Matcher::Never,
            behavior: (|u8| panic!("should not be called!")).into(),
        };

        assert_eq!(rule.called(&1), None);
    }

    #[test]
    fn called_returns_some() {
        let mut rule: Rule<u8, u8> = Rule {
            matcher: Matcher::Always,
            behavior: (|u| u + 1).into(),
        };

        assert_eq!(rule.called(&2), Some(3))
    }
}
