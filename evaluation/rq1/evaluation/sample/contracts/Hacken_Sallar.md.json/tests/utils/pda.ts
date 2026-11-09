import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { Sallar } from "../../target/types/sallar";
import { Program } from "@coral-xyz/anchor";

export const findProgramAddress = (key: string): [PublicKey, number] => {
    const program = anchor.workspace.Sallar as Program<Sallar>;
    let seed: Buffer[] = [Buffer.from(anchor.utils.bytes.utf8.encode(key))];

    const [_pda, _bump] = PublicKey.findProgramAddressSync(
        seed,
        program.programId,
    );

    return [_pda, _bump];
};
