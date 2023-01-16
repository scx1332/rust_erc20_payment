FROM ethereum/client-go as builder

FROM nikolaik/python-nodejs:python3.10-nodejs16
RUN apt-get update
RUN apt-get install -y vim

#install dependencies

RUN pip install web3 python-dotenv

WORKDIR /runtime/contracts-web3-create2
COPY contracts-web3-create2/package.json ./contracts-web3-create2/
COPY contracts-web3-create2/package-lock.json ./contracts-web3-create2/
RUN npm install

#copy geth from client-go
COPY --from=builder /usr/local/bin/geth /usr/local/bin/

#copy contracts and compile
COPY contracts-web3-create2/*.js ./contracts-web3-create2/
COPY contracts-web3-create2/contracts ./contracts-web3-create2/contracts
COPY contracts-web3-create2/scripts ./contracts-web3-create2/scripts
RUN npm run compile

#copy python scripts
WORKDIR /runtime
COPY *.py .
