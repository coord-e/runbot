use runbot::action::run;
use runbot::action::run_implicit;

pub trait CompileResult {
    fn compiler_message(&self) -> Option<&String>;
    fn program_message(&self) -> Option<&String>;
    fn status(&self) -> Option<u32>;
    fn signal(&self) -> Option<&String>;
    fn url(&self) -> Option<&String>;
}

impl CompileResult for run_implicit::Output {
    fn compiler_message(&self) -> Option<&String> {
        match self {
            run_implicit::Output::NoRun => None,
            run_implicit::Output::Run {
                compiler_message, ..
            } => compiler_message.as_ref(),
        }
    }

    fn program_message(&self) -> Option<&String> {
        match self {
            run_implicit::Output::NoRun => None,
            run_implicit::Output::Run {
                program_message, ..
            } => program_message.as_ref(),
        }
    }

    fn status(&self) -> Option<u32> {
        match self {
            run_implicit::Output::NoRun => None,
            run_implicit::Output::Run { status, .. } => *status,
        }
    }

    fn signal(&self) -> Option<&String> {
        match self {
            run_implicit::Output::NoRun => None,
            run_implicit::Output::Run { signal, .. } => signal.as_ref(),
        }
    }

    fn url(&self) -> Option<&String> {
        match self {
            run_implicit::Output::NoRun => None,
            run_implicit::Output::Run { url, .. } => url.as_ref(),
        }
    }
}

impl CompileResult for run::Output {
    fn compiler_message(&self) -> Option<&String> {
        self.compiler_message.as_ref()
    }

    fn program_message(&self) -> Option<&String> {
        self.program_message.as_ref()
    }

    fn status(&self) -> Option<u32> {
        self.status
    }

    fn signal(&self) -> Option<&String> {
        self.signal.as_ref()
    }

    fn url(&self) -> Option<&String> {
        self.url.as_ref()
    }
}
