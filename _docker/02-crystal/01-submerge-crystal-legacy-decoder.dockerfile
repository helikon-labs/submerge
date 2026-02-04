FROM node:24.13.0-trixie-slim AS build
RUN adduser --disabled-password --gecos "" submerge
WORKDIR /usr/src/app
COPY ./lib ./lib
COPY ./package.json ./
COPY ./tsconfig.json ./
COPY ./api.ts ./
COPY ./app.ts ./
RUN chown -R submerge:submerge /usr/src/app
USER submerge
RUN npm install \
    && npm run build
ENV RPC_URL="wss://rpc.ibp.network/polkadot"
ENV PORT=3000
EXPOSE 3000
CMD ["npm", "run", "start"]