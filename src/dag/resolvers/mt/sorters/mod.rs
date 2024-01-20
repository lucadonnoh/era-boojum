use std::sync::Arc;

use crate::{dag::{TrackId, guide::RegistrationNum, primitives::{ResolverIx, OrderIx}}, field::SmallField, cs::{Place, traits::cs::DstBuffer}, utils::PipeOp as _};

use super::{resolution_window::RWConfig, ResolverComms, ResolverCommonData};

pub mod sorter_playback;
pub mod sorter_runtime;


pub trait ResolverSortingMode<F: SmallField>: Sized
{
    type Arg;
    type Config: RWConfig<Self::TrackId> + 'static;
    type TrackId: TrackId + 'static;
    

    fn new(opts: Self::Arg, comms: Arc<ResolverComms>, debug_track: &Vec<Place>) -> (Self, Arc<ResolverCommonData<F, Self::TrackId>>);
    fn set_value(&mut self, key: Place, value: F);
    fn add_resolution<Fn>(&mut self, inputs: &[Place], outputs: &[Place], f: Fn)
    where
        Fn: FnOnce(&[F], &mut DstBuffer<'_, '_, F>) + Send + Sync;

    unsafe fn internalize(
        &mut self, 
        resolver_ix: ResolverIx,
        inputs: &[Place], 
        outputs: &[Place],
        added_at: RegistrationNum);
    fn internalize_one(
        &mut self,
        resolver_ix: ResolverIx,
        inputs: &[Place],
        outputs: &[Place],
        added_at: RegistrationNum
    ) -> Vec<ResolverIx>;

    fn flush(&mut self);
    fn final_flush(&mut self);
    fn write_sequence(&mut self);

    fn retrieve_sequence(&mut self) -> &ResolutionRecord;
}

#[derive(Default, Clone, Debug)]
pub struct ResolutionRecordItem { 
    added_at: RegistrationNum,
    accepted_at: RegistrationNum,
    /// The size of the order list when this registration was processed.
    order_len: usize,
    order_ix: OrderIx,
    parallelism: u16,
}

#[derive(Clone, Debug)]
pub struct ResolutionRecord {
    pub items: Vec<ResolutionRecordItem>,
    pub registrations_count: usize,
    pub values_count: usize
}

impl ResolutionRecord {
    fn new(registrations_count: usize, values_count: usize, size: usize) -> Self {
        Self {
            registrations_count,
            values_count,
            items:
                Vec::with_capacity(size)
                .op(|x| x.resize_with(size, ResolutionRecordItem::default))
        }
    }
}

pub trait ResolutionRecordWriter {
    fn store(&mut self, record: &ResolutionRecord);
}

pub trait ResolutionRecordSource {
    fn get(&self) -> &ResolutionRecord;
}
