# proving-server
![Github Actions](https://github.com/bind/proving-server/workflows/Tests/badge.svg)

ZK-Snark proving server built using Ark-Circom

**This project aims to be the go-to solution for quickly spinning up compute resources for generating snark proofs for zk applications.**

Inspired by https://zkga.me proof generation requirements, we imagine a world where there are many zk apps requiring bespoke proofs to be computed. 
Consumer devices will continue to become powerful and proof generation less resource intensive, which will hopefully quickly make this project obselete.
In the meantime,some proofs might have too many constraints to run on the current generation of devices, or power-users of these apps might look to move zk-proof computation on to dedicated hardware.


The service exposes three endpoints:

- Create Prover
- Check Prover Status
- Generate Proof



How you can help (Things I haven't gotten to yet):
- [ ] Build out a dockerfile
- [ ] Guide to hosting on Google Cloud Run
- [ ] Update the remote snarking df plugin
