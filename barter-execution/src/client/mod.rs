use crate::{
    balance::AssetBalance,
    error::{UnindexedClientError, UnindexedOrderError},
    order::{
        state::{Cancelled, Open},
        Order, RequestCancel, RequestOpen,
    },
    trade::Trade,
    UnindexedAccountEvent, UnindexedAccountSnapshot,
};
use barter_instrument::{
    asset::{name::AssetNameExchange, QuoteAsset},
    exchange::ExchangeId,
    instrument::name::InstrumentNameExchange,
};
use chrono::{DateTime, Utc};
use futures::Stream;
use std::future::Future;

mod binance;
pub mod mock;

pub trait ExecutionClient
where
    Self: Clone,
{
    const EXCHANGE: ExchangeId;

    type Config: Clone;
    type AccountStream: Stream<Item = UnindexedAccountEvent>;

    fn new(config: Self::Config) -> Self;

    fn account_snapshot(
        &self,
        assets: &[AssetNameExchange],
        instruments: &[InstrumentNameExchange],
    ) -> impl Future<Output = Result<UnindexedAccountSnapshot, UnindexedClientError>> + Send;

    fn account_stream(
        &self,
        assets: &[AssetNameExchange],
        instruments: &[InstrumentNameExchange],
    ) -> impl Future<Output = Result<Self::AccountStream, UnindexedClientError>> + Send;

    fn cancel_order(
        &self,
        request: Order<ExchangeId, &InstrumentNameExchange, RequestCancel>,
    ) -> impl Future<
        Output = Order<ExchangeId, InstrumentNameExchange, Result<Cancelled, UnindexedOrderError>>,
    > + Send;

    fn cancel_orders<'a>(
        &self,
        requests: impl IntoIterator<Item = Order<ExchangeId, &'a InstrumentNameExchange, RequestCancel>>,
    ) -> impl Stream<
        Item = Order<ExchangeId, InstrumentNameExchange, Result<Cancelled, UnindexedOrderError>>,
    > {
        futures::stream::FuturesUnordered::from_iter(
            requests
                .into_iter()
                .map(|request| self.cancel_order(request)),
        )
    }

    fn open_order(
        &self,
        request: Order<ExchangeId, &InstrumentNameExchange, RequestOpen>,
    ) -> impl Future<
        Output = Order<ExchangeId, InstrumentNameExchange, Result<Open, UnindexedOrderError>>,
    > + Send;

    fn open_orders<'a>(
        &self,
        requests: impl IntoIterator<Item = Order<ExchangeId, &'a InstrumentNameExchange, RequestOpen>>,
    ) -> impl Stream<Item = Order<ExchangeId, InstrumentNameExchange, Result<Open, UnindexedOrderError>>>
    {
        futures::stream::FuturesUnordered::from_iter(
            requests.into_iter().map(|request| self.open_order(request)),
        )
    }

    fn fetch_balances(
        &self,
    ) -> impl Future<Output = Result<Vec<AssetBalance<AssetNameExchange>>, UnindexedClientError>>;

    fn fetch_open_orders(
        &self,
    ) -> impl Future<
        Output = Result<Vec<Order<ExchangeId, InstrumentNameExchange, Open>>, UnindexedClientError>,
    >;

    fn fetch_trades(
        &self,
        time_since: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<Trade<QuoteAsset, InstrumentNameExchange>>, UnindexedClientError>>;
}
