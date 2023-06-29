use async_trait::async_trait;
use miette::Result;

pub struct ProcessingChain<S1: ProcessingStep, S2: ProcessingStep<Input = S1::Output>>(S1, S2);
pub struct ProcessingPipeline<I: Send + Sync, O: Send + Sync>(
    Box<dyn ProcessingStep<Input = I, Output = O>>,
);

#[async_trait]
pub trait ProcessingStep: Send + Sync {
    type Input: Send + Sync;
    type Output: Send + Sync;

    async fn process(&self, input: Self::Input) -> Result<Self::Output>;
}

#[async_trait]
impl<S1: ProcessingStep, S2: ProcessingStep<Input = S1::Output>> ProcessingStep
    for ProcessingChain<S1, S2>
{
    type Input = S1::Input;
    type Output = S2::Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        let first = self.0.process(input).await?;
        self.1.process(first).await
    }
}

pub trait ProcessingStepChain: Sized + ProcessingStep {
    fn chain<S: ProcessingStep<Input = Self::Output>>(self, other: S) -> ProcessingChain<Self, S> {
        ProcessingChain(self, other)
    }
}

impl<S: ProcessingStep> ProcessingStepChain for S {}

pub trait IntoPipeline: Sized + ProcessingStep + 'static {
    fn into_pipeline(self) -> ProcessingPipeline<Self::Input, Self::Output> {
        ProcessingPipeline(Box::new(self))
    }
}

impl<S: ProcessingStep + 'static> IntoPipeline for S {}

#[async_trait]
impl<I: Send + Sync, O: Send + Sync> ProcessingStep for ProcessingPipeline<I, O> {
    type Input = I;
    type Output = O;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        self.0.process(input).await
    }
}
