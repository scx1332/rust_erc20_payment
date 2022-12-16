FROM ethereum/client-go as builder

FROM nikolaik/python-nodejs:python3.10-nodejs16
RUN apt-get update
RUN apt-get install -y vim

COPY --from=builder /usr/local/bin/geth /usr/local/bin/

WORKDIR /runtime
RUN pip install web3 dotenv
COPY *.py .
COPY contracts-web3-create2/*.json ./contracts-web3-create2/
COPY contracts-web3-create2/*.js ./contracts-web3-create2/
COPY contracts-web3-create2/contracts ./contracts-web3-create2/contracts
COPY contracts-web3-create2/scripts ./contracts-web3-create2/scripts

WORKDIR /runtime/contracts-web3-create2
RUN npm install
RUN npm run compile

WORKDIR /runtime