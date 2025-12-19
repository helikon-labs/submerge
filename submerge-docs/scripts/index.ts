const CHAIN = 'coretime-polkadot';
const BASE_URL = `https://${CHAIN}.crystal.submerge.io/api/v1`;

type MetaData = {
    data: { specVersion: number; metadataVersion: number }[];
    pagination: { page: number; pageSize: number; total: number };
};

type Events = {
    pagination: { page: number; pageSize: number; total: number };
};

const main = async () => {
    const metadata: MetaData = await (await fetch(`${BASE_URL}/metadata?page_size=100`)).json();

    for (const { specVersion } of metadata.data) {
        const [palletName, palletEventName] = 'xcmpqueue.XcmpMessageSent'.split('.');
        const events = await (
            await fetch(
                `${BASE_URL}/events?pallet_name=${palletName}&event_name=${palletEventName}&min_spec_version=${specVersion}&max_spec_version=${specVersion}&include_args=true&page_size=1`,
            )
        ).json();
        console.log(events.data?.at(0)?.args);
    }
};

main()
    .then(() => process.exit(0))
    .catch(console.error);
