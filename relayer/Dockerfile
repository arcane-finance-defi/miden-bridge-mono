FROM --platform=linux/amd64 node:20 as builder

WORKDIR /usr/src/app

COPY ./evm-side/package.json ./
COPY ./evm-side/package-lock.json ./

RUN npm ci

WORKDIR /usr/src/api

COPY ./api/package.json ./
COPY ./api/package-lock.json ./

RUN npm ci
COPY ./api/ .
RUN npm run gen

WORKDIR /usr/src/app

COPY ./evm-side/. .

RUN npm run build && npm prune --production

##################################################################

FROM --platform=linux/amd64 node:20-slim as backend
RUN npm install tsx -g

WORKDIR /usr/src/app

COPY ./evm-side/scripts/. ./scripts

RUN chmod +x ./scripts/*.sh

COPY --from=builder /usr/src/app/package.json ./package.json 
COPY --from=builder /usr/src/app/node_modules/. ./node_modules/
COPY --from=builder /usr/src/app/dist/. ./

ENTRYPOINT [ "./scripts/prod.sh" ]
