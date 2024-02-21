# What I have done
- Admin adds or deletes supported token.
- User deposit or withdraw token.
- Record of all users' balance of all kinds of tokens.

# What I have not done
- Signature verification & user account balance modification, not familar with solana's mechanism and running out of time 
- Build .so. My WSL2 on windows11 has network issues which lead to timeout of `sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`. Install binary `solana` & `cargo-build-bpf` only will lead to build error `Failed to execute /root/.cargo/bin/cargo-build-sbf: No such file or directory`.
- deploy contract. I have created solana account `D6gQXdUX7AwrGtdQaCuZ5p1MwyXHaidWvKypdKY9bmkA` and got 5 sols in devnet. But `solana program deploy ./target/deploy/hello_world.so`(fake .so) always timeout due to network issue.  

In summary, these undone tasks mainly due to 3 reasons:
- Bad network
- New to solana
- Limited time to solve above problems
