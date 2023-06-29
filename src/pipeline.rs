use async_trait::async_trait;
use futures::future;
use miette::Result;

/// The result of combining two processing steps
pub struct Chained<S1: ProcessingStep, S2: ProcessingStep<Input = S1::Output>>(S1, S2);

/// An adapter to execute a step with multiple inputs in parallel
pub struct Parallel<S: ProcessingStep>(S);

/// An adapter to map the result of the pipeline
pub struct Map<S: ProcessingStep, T: Send + Sync>(S, Box<dyn Fn(S::Output) -> T + Send + Sync>);

/// An adapter to dynamically construct the next step mapper depending on the previous one
pub struct Construct<S1: ProcessingStep, S2: ProcessingStep<Input = T>, T>(
    S1,
    Box<dyn Fn(S1::Output) -> (T, S2) + Send + Sync>,
);

/// A generic wrapper for processing pipelines
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
    for Chained<S1, S2>
{
    type Input = S1::Input;
    type Output = S2::Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        let first = self.0.process(input).await?;
        self.1.process(first).await
    }
}

#[async_trait]
impl<S: ProcessingStep> ProcessingStep for Parallel<S> {
    type Input = Vec<S::Input>;
    type Output = Vec<S::Output>;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        future::try_join_all(input.into_iter().map(|i| self.0.process(i))).await
    }
}

pub trait ProcessingChain: Sized + ProcessingStep {
    fn chain<S: ProcessingStep<Input = Self::Output>>(self, other: S) -> Chained<Self, S> {
        Chained(self, other)
    }
}

impl<S: ProcessingStep> ProcessingChain for S {}

pub trait ProcessingParallel: Sized + ProcessingStep {
    fn parallel(self) -> Parallel<Self> {
        Parallel(self)
    }
}

impl<S: ProcessingStep> ProcessingParallel for S {}

pub trait IntoPipeline: Sized + ProcessingStep + 'static {
    fn into_pipeline(self) -> ProcessingPipeline<Self::Input, Self::Output> {
        ProcessingPipeline(Box::new(self))
    }
}

pub trait ProcessingMap: ProcessingStep + Sized {
    fn map<F: Fn(Self::Output) -> T + Send + Sync + 'static, T: Send + Sync>(
        self,
        map_fn: F,
    ) -> Map<Self, T> {
        Map(self, Box::new(map_fn))
    }
}

impl<S: ProcessingStep> ProcessingMap for S {}

#[async_trait]
impl<S: ProcessingStep, T: Send + Sync> ProcessingStep for Map<S, T> {
    type Input = S::Input;
    type Output = T;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        let inner_result = self.0.process(input).await?;

        Ok(self.1(inner_result))
    }
}

pub trait ProcessingConstruct: ProcessingStep + Sized {
    fn construct<
        F: Fn(Self::Output) -> (T, S) + Send + Sync + 'static,
        S: ProcessingStep<Input = T>,
        T: Send + Sync,
    >(
        self,
        construct_fn: F,
    ) -> Construct<Self, S, T> {
        Construct(self, Box::new(construct_fn))
    }
}

impl<S: ProcessingStep> ProcessingConstruct for S {}

#[async_trait]
impl<S1: ProcessingStep, S2: ProcessingStep<Input = T>, T: Send + Sync> ProcessingStep
    for Construct<S1, S2, T>
{
    type Input = S1::Input;
    type Output = S2::Output;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        let inner_output = self.0.process(input).await?;
        let (new_input, step) = self.1(inner_output);

        step.process(new_input).await
    }
}

#[async_trait]
impl<I: Send + Sync, O: Send + Sync> ProcessingStep for ProcessingPipeline<I, O> {
    type Input = I;
    type Output = O;

    async fn process(&self, input: Self::Input) -> Result<Self::Output> {
        self.0.process(input).await
    }
}

impl<S: ProcessingStep + 'static> IntoPipeline for S {}
