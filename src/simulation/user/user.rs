use std::time::Duration;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use rand::Rng;
use rand_distr::Exp;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery, UserAction};
use crate::simulation::user::UserParameters;

pub struct User {
    config: UserParameters,
    sale_distribution: Exp<f64>,
    edit_distribution: Exp<f64>,
    pub(crate) action_output: Output<PlatformQuery>,
}
impl User {
    pub fn new(config: UserParameters) -> User {
        User {
            sale_distribution: Exp::new(config.average_sales_per_hour / 60.0 / 60.0 / 1000.0).unwrap(), // Time Between Sales in milliseconds
            edit_distribution: Exp::new(config.average_edits_per_day / 24.0 / 60.0 / 60.0 / 1000.0).unwrap(), // Time Between Edits in milliseconds.
            config,
            action_output: Default::default(),
        }
    }

    pub fn do_edit<'a>( // Messy signature for self-scheduling functions. Is NexoSim Limitation.
                        &'a mut self,
                        _: (),
                        ctx: &'a mut Context<Self>
    ) -> impl Future<Output=()> + Send + 'a {
        async move {
            // Do Sale
            self.action_output.send(PlatformQuery::User(UserAction::Assignment(self.config.edit_to))).await;
            // Schedule next sale
            let next_edit = ctx.time() + Duration::from_millis(rand::rng().sample(self.edit_distribution).round() as u64);
            ctx.schedule_event(next_edit, Self::do_edit, ()).unwrap();
        }
    }

    pub fn do_sale<'a>( // Messy signature for self-scheduling functions. Is NexoSim Limitation.
        &'a mut self,
        _: (),
        ctx: &'a mut Context<Self>
    ) -> impl Future<Output=()> + Send + 'a {
        async move {
            // Do Sale
            self.action_output.send(PlatformQuery::User(UserAction::Mutation(-1))).await;
            // Schedule next sale
            let next_sale = ctx.time() + Duration::from_millis(rand::rng().sample(self.sale_distribution).round() as u64);
            ctx.schedule_event(next_sale, Self::do_sale, ()).unwrap();
        }
    }
}

impl Model for User {
    async fn init(self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        // Schedule first sale after start time. (For observation of deviation.)
        let first_sale = ctx.time()
            + self.config.start_after
            + Duration::from_millis(rand::rng().sample(self.sale_distribution).round() as u64);
        ctx.schedule_event(first_sale, Self::do_sale, ()).unwrap();

        let first_edit = ctx.time()
            + self.config.start_after
            + Duration::from_millis(rand::rng().sample(self.edit_distribution).round() as u64);
        ctx.schedule_event(first_edit, Self::do_edit, ()).unwrap();


        self.into()
    }
}