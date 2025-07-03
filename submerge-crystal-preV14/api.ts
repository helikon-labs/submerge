import express, { Application, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { StatusCodes } from 'http-status-codes';
import { GenericExtrinsic } from '@polkadot/types';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Registry } from '@polkadot/types/types';

class API {
    private readonly app: Application;
    private api!: ApiPromise;
    private registryCache: Map<number, Registry>;

    constructor() {
        this.app = express();
        this.app.set('trust proxy', true);
        this.app.use(express.json());
        this.app.use(express.urlencoded({ extended: true }));
        this.app.use(cors());
        this.app.use(helmet());
        this.registryCache = new Map<number, Registry>();
    }

    async setup() {
        const wsProvider = new WsProvider('wss://rpc.helikon.io/kusama');
        this.api = await ApiPromise.create({ provider: wsProvider });
        await this.api.isReady;
        const router = express.Router();
        router.post('/event', async (request, response) => {
            await this.decodeEvent(request, response);
        });
        router.post('/extrinsic', async (request, response) => {
            await this.decodeExtrinsic(request, response);
        });
        this.app.use('/decode', router);
    }

    async start(port: number) {
        await this.setup();
        this.app.listen(port, () => {
            console.log(`Faucet server is listening on port ${port}.`);
        });
    }

    private async getRegistry(blockHash: string, specVersion: number): Promise<Registry> {
        if (!this.registryCache.has(specVersion)) {
            console.log(specVersion, 'create');
            let api = await this.api.at(blockHash);
            this.registryCache.set(specVersion, api.registry);
        }
        return this.registryCache.get(specVersion)!;
    }

    private async decodeEvent(request: Request, response: Response) {
        try {
            const { blockHash, specVersion, hex } = request.body;
            if (!blockHash) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Block hash not found in the request body.' });
            }
            if (!specVersion) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Spec version not found in the request body.' });
            }
            if (!hex) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Extrinsic hex string not found in the request body.' });
            }

            const registry = await this.getRegistry(blockHash, specVersion);
            const event = registry.createType('EventRecord', hex);
            return response.status(StatusCodes.OK).json(event.toHuman());
        } catch (error) {
            return response.status(StatusCodes.BAD_REQUEST).json({
                error: `${error}`
            });
        }
    }

    private async decodeExtrinsic(request: Request, response: Response) {
        try {
            const { blockHash, specVersion, hex } = request.body;
            if (!blockHash) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Block hash not found in the request body.' });
            }
            if (!specVersion) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Spec version not found in the request body.' });
            }
            if (!hex) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Extrinsic hex string not found in the request body.' });
            }
            const registry = await this.getRegistry(blockHash, specVersion);
            const extrinsic = new GenericExtrinsic(registry, hex);
            return response.status(StatusCodes.OK).json(extrinsic.toHuman());
        } catch (error) {
            return response.status(StatusCodes.BAD_REQUEST).json({
                error: `${error}`
            });
        }
    }
}

export { API };
