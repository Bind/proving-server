# proving-server (Beta)
![Github Actions](https://github.com/bind/proving-server/workflows/Tests/badge.svg)

ZK-Snark proving server built using Ark-Circom

**This project aims to be the go-to solution for quickly spinning up compute resources for generating snark proofs for zk applications.**

Inspired by https://zkga.me proof generation requirements, we imagine a world where there are many zk apps requiring bespoke proofs to be computed. 
Consumer devices will continue to become more powerful and proof generation less resource intensive, which should make this project obselete.
In the meantime, some proofs might have too many constraints to run on the current generation of devices, or power-users of these apps might want to move zk-proof computation on to dedicated hardware. We hope to bridge that temporary divide and help zk-app developers and users push beyond the current client limitations.


## The basics


The service exposes three endpoints:

- Create Prover
```
curl --request POST \
  --url http://localhost:8000/v1/prover \
  --header 'Content-Type: application/json' \
  --data '{
	"name":"move",
	"version":"3",
	"path_to_wasm": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm",
	"path_to_r1cs": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs",
	"path_to_zkey": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey",
	"builder_params": [
          "x1",
          "y1",
          "x2",
          "y2",
          "r",
          "distMax",
          "PLANETHASH_KEY",
          "SPACETYPE_KEY",
          "SCALE",
          "xMirror",
          "yMirror"
         ]
}'
```

- Check Prover Status
```
curl --request GET \
  --url http://localhost:8000/v1/prover/<prover_name>/<version>
```

- Generate Proof
```
curl --request POST \
  --url http://localhost:8000/v1/prove/<prover_name>/<version> \
  --header 'Content-Type: application/json' \
  --data '{
    "x1":100,
    "y1":100,
    "x2":120,
    "y2":120,
    "r":8000,
    "distMax":29,
    "PLANETHASH_KEY":1729,
    "SPACETYPE_KEY":1730,
    "SCALE":16384,
    "xMirror":0,
    "yMirror":0
  }'
```


## How this works

When hitting the `/prover/` endpoint, a job is scheduled to fetch the provided wasm, zkey, and r1cs files and then instantiate a circom circuit. After that job has completed you can than call the `prove/<name>/<version>` endpoint with the required input parameters and get the proof back.
	
We use a basic sqlite in memory database to facilitate job tracking right now, which has its trade-offs. If there is enough excitement or demand we can quickly integrate an external db like psql.

Otherwise this app makes heavy use of the work done by contributors to https://github.com/gakonst/ark-circom and would quite literally not work without them!

### Things I haven't gotten to yet

- [ ] Build out a dockerfile
- [ ] Guide to hosting on Google Cloud Run

### Inspiration

This project was heavily inspired by the work done by the good folks at https://github.com/projectsophon & https://github.com/darkforest-eth/
