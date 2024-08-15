use std::io;

pub trait Output {
    fn out(&mut self) -> impl io::Write;
    fn err(&mut self) -> impl io::Write;
}

pub struct SingleOutput<Out>(pub(crate) Out);

impl<Out> SingleOutput<Out> {
    pub fn new(out: Out) -> Self {
        Self(out)
    }
}

impl<Out> Output for SingleOutput<Out>
where
    Out: io::Write,
{
    fn out(&mut self) -> impl io::Write {
        &mut self.0
    }

    fn err(&mut self) -> impl io::Write {
        &mut self.0
    }
}

pub struct SplitOutput<Out, Err>(pub(crate) Out, pub(crate) Err);

impl<Out, Err> SplitOutput<Out, Err> {
    pub fn new(out: Out, err: Err) -> Self {
        Self(out, err)
    }
}

impl<Out, Err> Output for SplitOutput<Out, Err>
where
    Out: io::Write,
    Err: io::Write,
{
    fn out(&mut self) -> impl io::Write {
        &mut self.0
    }

    fn err(&mut self) -> impl io::Write {
        &mut self.1
    }
}
