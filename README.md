# proving-server
![Github Actions](https://github.com/bind/proving-server/workflows/Tests/badge.svg)

ZK-Snark proving server built using Ark-Circom

**This project aims to be the go-to solution for quickly spinning up compute resources for generating snark proofs for zk applications.**

Inspired by https://zkga.me proof generation requirements, we imagine a world where there are many zk apps requiring bespoke proofs to be computed. 
Consumer devices will continue to become more powerful and proof generation less resource intensive, which will hopefully quickly make this project obselete.
In the meantime, some proofs might have too many constraints to run on the current generation of devices, or power-users of these apps might want to move zk-proof computation on to dedicated hardware.


The service exposes three endpoints:

- Create Prover
```
curl --request POST \
  --url http://localhost:8000/v1/prover \
  --header 'Content-Type: application/json' \
  --data '{ \
	"name":"move", \
	"version":"3", \
	"path_to_wasm": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm", \
	"path_to_r1cs": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs", \
	"path_to_zkey": "https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey", \
	"builder_params": [ \
          "x1", \
          "y1", \
          "x2", \
          "y2", \
          "r", \
          "distMax", \
          "PLANETHASH_KEY", \
          "SPACETYPE_KEY",  \
          "SCALE",  \
          "xMirror", \
          "yMirror" \
         ] \
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
  --data '{ \
    "x1":100, \
    "y1":100, \
    "x2":120, \
    "y2":120, \
    "r":8000, \
    "distMax":29, \
    "PLANETHASH_KEY":1729, \
    "SPACETYPE_KEY":1730, \
    "SCALE":16384, \
    "xMirror":0, \
    "yMirror":0 \
  }'
```


Things I haven't gotten to yet:
- [ ] Build out a dockerfile
- [ ] Guide to hosting on Google Cloud Run
