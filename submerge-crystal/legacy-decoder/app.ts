import { API } from './api';
import * as dotevnv from 'dotenv';

dotevnv.config();

if (!process.env.PORT) {
    console.error('API port not set in the environment. Exiting.');
    process.exit();
}

if (!process.env.RPC_URL) {
    console.error('Substrate RPC URL not set in the environment. Exiting.');
    process.exit();
}

const port = parseInt(process.env.PORT as string);
const rpcURL = process.env.RPC_URL as string;

new API().start(port, rpcURL);
