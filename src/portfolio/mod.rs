use crate::{
    data::MarketMeta,
    event::Event,
    execution::FillEvent,
    portfolio::{error::PortfolioError, position::PositionUpdate},
    strategy::{Decision, Signal, SignalForceExit},
};
use barter_data::event::{DataKind, MarketEvent};
use barter_integration::model::{Exchange, Instrument};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Logic for [`OrderEvent`] quantity allocation.
pub mod allocator;

/// Barter portfolio module specific errors.
pub mod error;

/// Core Portfolio logic containing an implementation of [`MarketUpdater`],
/// [`OrderGenerator`] and [`FillUpdater`]. Utilises the risk and allocator logic to optimise
/// [`OrderEvent`] generation.
pub mod portfolio;

/// Data structures encapsulating the state of a trading [`Position`](position::Position), as
/// well as the logic for entering, updating and exiting them.
pub mod position;

/// Repositories for persisting Portfolio state.
pub mod repository;

/// Logic for evaluating the risk associated with a proposed [`OrderEvent`].
pub mod risk;

/// Updates the Portfolio from an input [`MarketEvent`].
pub trait MarketUpdater {
    /// Determines if the Portfolio has an open Position relating to the input [`MarketEvent`]. If
    /// so it updates it using the market data, and returns a [`PositionUpdate`] detailing the
    /// changes.
    fn update_from_market(
        &mut self,
        market: &MarketEvent<DataKind>,
    ) -> Result<Option<PositionUpdate>, PortfolioError>;
}

/// May generate an [`OrderEvent`] from an input advisory [`Signal`].
pub trait OrderGenerator {
    /// May generate an [`OrderEvent`] after analysing an input advisory [`Signal`].
    fn generate_order(&mut self, signal: &Signal) -> Result<Option<OrderEvent>, PortfolioError>;

    /// Generates an exit [`OrderEvent`] if there is an open [`Position`](position::Position)
    /// associated with the input [`SignalForceExit`]'s [`PositionId`](position::PositionId).
    fn generate_exit_order(
        &mut self,
        signal: SignalForceExit,
    ) -> Result<Option<OrderEvent>, PortfolioError>;
}

/// Updates the Portfolio from an input [`FillEvent`].
pub trait FillUpdater {
    /// Updates the Portfolio state using the input [`FillEvent`]. The [`FillEvent`] triggers a
    /// Position entry or exit, and the Portfolio updates key fields such as current_cash and
    /// current_value accordingly.
    fn update_from_fill(&mut self, fill: &FillEvent) -> Result<Vec<Event>, PortfolioError>;
}

/// Orders are generated by the portfolio and details work to be done by an Execution handler to
/// open a trade.
#[derive(Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct OrderEvent {
    pub time: DateTime<Utc>,
    pub exchange: Exchange,
    pub instrument: Instrument,
    /// Metadata propagated from source MarketEvent
    pub market_meta: MarketMeta,
    /// LONG, CloseLong, SHORT or CloseShort
    pub decision: Decision,
    /// +ve or -ve Quantity depending on Decision
    pub quantity: f64,
    /// MARKET, LIMIT etc
    pub order_type: OrderType,
}

impl OrderEvent {
    pub const ORGANIC_ORDER: &'static str = "Order";
    pub const FORCED_EXIT_ORDER: &'static str = "OrderForcedExit";

    /// Returns a OrderEventBuilder instance.
    pub fn builder() -> OrderEventBuilder {
        OrderEventBuilder::new()
    }
}

/// Type of order the portfolio wants the execution::handler to place.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub enum OrderType {
    Market,
    Limit,
    Bracket,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::Market
    }
}

/// Builder to construct OrderEvent instances.
#[derive(Debug, Default)]
pub struct OrderEventBuilder {
    pub time: Option<DateTime<Utc>>,
    pub exchange: Option<Exchange>,
    pub instrument: Option<Instrument>,
    pub market_meta: Option<MarketMeta>,
    pub decision: Option<Decision>,
    pub quantity: Option<f64>,
    pub order_type: Option<OrderType>,
}

impl OrderEventBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time(self, value: DateTime<Utc>) -> Self {
        Self {
            time: Some(value),
            ..self
        }
    }

    pub fn exchange(self, value: Exchange) -> Self {
        Self {
            exchange: Some(value),
            ..self
        }
    }

    pub fn instrument(self, value: Instrument) -> Self {
        Self {
            instrument: Some(value),
            ..self
        }
    }

    pub fn market_meta(self, value: MarketMeta) -> Self {
        Self {
            market_meta: Some(value),
            ..self
        }
    }

    pub fn decision(self, value: Decision) -> Self {
        Self {
            decision: Some(value),
            ..self
        }
    }

    pub fn quantity(self, value: f64) -> Self {
        Self {
            quantity: Some(value),
            ..self
        }
    }

    pub fn order_type(self, value: OrderType) -> Self {
        Self {
            order_type: Some(value),
            ..self
        }
    }

    pub fn build(self) -> Result<OrderEvent, PortfolioError> {
        Ok(OrderEvent {
            time: self.time.ok_or(PortfolioError::BuilderIncomplete("time"))?,
            exchange: self
                .exchange
                .ok_or(PortfolioError::BuilderIncomplete("exchange"))?,
            instrument: self
                .instrument
                .ok_or(PortfolioError::BuilderIncomplete("instrument"))?,
            market_meta: self
                .market_meta
                .ok_or(PortfolioError::BuilderIncomplete("market_meta"))?,
            decision: self
                .decision
                .ok_or(PortfolioError::BuilderIncomplete("decision"))?,
            quantity: self
                .quantity
                .ok_or(PortfolioError::BuilderIncomplete("quantity"))?,
            order_type: self
                .order_type
                .ok_or(PortfolioError::BuilderIncomplete("order_type"))?,
        })
    }
}

/// Communicates a String represents a unique identifier for an Engine's Portfolio [`Balance`].
pub type BalanceId = String;

/// Total and available balance at a point in time.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Serialize)]
pub struct Balance {
    pub time: DateTime<Utc>,
    pub total: f64,
    pub available: f64,
}

impl Default for Balance {
    fn default() -> Self {
        Self {
            time: Utc::now(),
            total: 0.0,
            available: 0.0,
        }
    }
}

impl Balance {
    /// Construct a new [`Balance`] using the provided total & available balance values.
    pub fn new(time: DateTime<Utc>, total: f64, available: f64) -> Self {
        Self {
            time,
            total,
            available,
        }
    }

    /// Returns the unique identifier for an Engine's [`Balance`].
    pub fn balance_id(engine_id: Uuid) -> BalanceId {
        format!("{}_balance", engine_id)
    }
}
