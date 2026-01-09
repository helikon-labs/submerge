import {
  BlockService,
  GenesisService,
  EventService,
  MetadataService,
} from "./client";

const main = async () => {
  const [
    { data: blocks },
    { data: genesisRecords },
    { data: events },
    { data: metadataList },
  ] = await Promise.all([
    BlockService.getExtrinsicsByBlockReferenceAndIndex({
      path: { block_ref: "234", extrinsic_index: 0 },
    }),
    GenesisService.getGenesisRecords(),
    EventService.getEvents(),
    MetadataService.getMetadataList(),
  ]);

  console.log(blocks, genesisRecords, events, metadataList);
};

main().catch(console.error);
