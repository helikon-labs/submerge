import express, { Application, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { StatusCodes } from 'http-status-codes';
import { GenericExtrinsic } from '@polkadot/types';
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Registry } from '@polkadot/types/types';
import { LRUCache } from 'lru-cache';

class API {
    private readonly app: Application;
    private api!: ApiPromise;
    private registryCache: LRUCache<number, Registry>;

    constructor() {
        this.app = express();
        this.app.set('trust proxy', true);
        this.app.use(express.json({ limit: '100mb' }));
        this.app.use(express.urlencoded({ extended: true }));
        this.app.use(cors());
        this.app.use(helmet());
        this.registryCache = new LRUCache<number, Registry>({
            max: 10,
        });
    }

    async setup(rpcURL: string) {
        const wsProvider = new WsProvider(rpcURL);
        this.api = await ApiPromise.create({ provider: wsProvider });
        await this.api.isReady;
        const router = express.Router();
        router.post('/event', async (request, response) => {
            await this.decodeEvent(request, response);
        });
        router.post('/events', async (request, response) => {
            await this.decodeEvents(request, response);
        });
        router.post('/extrinsic', async (request, response) => {
            await this.decodeExtrinsic(request, response);
        });
        router.post('/block-weight', async (request, response) => {
            await this.decodeBlockWeight(request, response);
        });
        router.post('/type', async (request, response) => {
            await this.decodeType(request, response);
        });
        this.app.use('/decode', router);
    }

    async start(port: number, rpcURL: string) {
        await this.setup(rpcURL);
        this.app.listen(port, () => {
            console.log(`Submerge legacy decode server is initialized with RPC URL ${rpcURL}.`);
            console.log(`Listening on port ${port}.`);
        });
    }

    private async getRegistry(blockHash: string, specVersion: number): Promise<Registry> {
        if (!this.registryCache.has(specVersion)) {
            console.log(`Registry cached for spec version ${specVersion}.`);
            const api = await this.api.at(blockHash);
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
                    .json({ error: 'Extrinsic hexadecimal string not found in the request body.' });
            }

            const registry = await this.getRegistry(blockHash, specVersion);
            const event = registry.createType('EventRecord', hex);
            return response.status(StatusCodes.OK).json(event.toHuman());
        } catch (error) {
            return response.status(StatusCodes.INTERNAL_SERVER_ERROR).json({
                error: `${error}`,
            });
        }
    }

    private async decodeEvents(request: Request, response: Response) {
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
                    .json({ error: 'Extrinsic hexadecimal string not found in the request body.' });
            }
            const registry = await this.getRegistry(blockHash, specVersion);
            const events = registry.createType('Vec<EventRecord>', hex);
            return response.status(200).json(events.toHuman());
        } catch (error) {
            return response.status(StatusCodes.INTERNAL_SERVER_ERROR).json({
                error: `${error}`,
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
                    .json({ error: 'Extrinsic hexadecimal string not found in the request body.' });
            }
            const registry = await this.getRegistry(blockHash, specVersion);
            const extrinsic = new GenericExtrinsic(registry, hex);
            return response.status(StatusCodes.OK).json(extrinsic.toHuman());
        } catch (error) {
            return response.status(StatusCodes.INTERNAL_SERVER_ERROR).json({
                error: `${error}`,
            });
        }
    }

    private async decodeBlockWeight(request: Request, response: Response) {
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
                    .json({ error: 'Extrinsic hexadecimal string not found in the request body.' });
            }

            const registry = await this.getRegistry(blockHash, specVersion);
            const weight = registry.createType('PerDispatchClassWeight', hex);
            return response.status(StatusCodes.OK).json(weight.toHuman());
        } catch (error) {
            return response.status(StatusCodes.INTERNAL_SERVER_ERROR).json({
                error: `${error}`,
            });
        }
    }

    private async decodeType(request: Request, response: Response) {
        try {
            const { blockHash, specVersion, typeName, hex } = request.body;
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
            if (!typeName) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Type name not found in the request body.' });
            }
            if (!hex) {
                return response
                    .status(StatusCodes.BAD_REQUEST)
                    .json({ error: 'Extrinsic hexadecimal string not found in the request body.' });
            }

            const registry = await this.getRegistry(blockHash, specVersion);
            const weight = registry.createType(typeName, hex);
            return response.status(StatusCodes.OK).json(weight.toHuman());
        } catch (error) {
            return response.status(StatusCodes.INTERNAL_SERVER_ERROR).json({
                error: `${error}`,
            });
        }
    }
}

export { API };
