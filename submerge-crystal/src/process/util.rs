use parity_scale_codec::Decode;
use submerge_base::types::substrate::block_trace::BlockTrace;
use submerge_util::substrate::storage::get_storage_plain_key;

pub fn get_extrinsic_count(trace: &BlockTrace) -> anyhow::Result<u32> {
    let extrinsic_count_key = get_storage_plain_key("System", "ExtrinsicCount");
    let mut extrinsic_count: u32 = 0;
    for trace in trace.events.iter() {
        let trace_data = &trace.data_wrapper.data;
        if trace_data.key == extrinsic_count_key && trace_data.value.to_lowercase() != "none" {
            let value = trace_data
                .value
                .trim_start_matches("Some(")
                .trim_end_matches(")");
            let mut bytes: &[u8] = &hex::decode(value)?;
            extrinsic_count = Decode::decode(&mut bytes)?;
        }
    }
    Ok(extrinsic_count)
}

pub fn get_event_count(trace: &BlockTrace) -> anyhow::Result<u32> {
    let event_count_key = get_storage_plain_key("System", "EventCount");
    let mut event_count: u32 = 0;
    for trace in trace.events.iter() {
        let trace_data = &trace.data_wrapper.data;
        if trace_data.key == event_count_key && trace_data.value.to_lowercase() != "none" {
            let value = trace_data
                .value
                .trim_start_matches("Some(")
                .trim_end_matches(")");
            let mut bytes: &[u8] = &hex::decode(value)?;
            event_count = Decode::decode(&mut bytes)?;
        }
    }
    Ok(event_count)
}
