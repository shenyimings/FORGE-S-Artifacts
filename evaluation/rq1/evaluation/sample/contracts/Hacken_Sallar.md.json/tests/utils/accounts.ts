import * as anchor from "@coral-xyz/anchor";
import {
    createAssociatedTokenAccountInstruction, getAssociatedTokenAddress
} from "@solana/spl-token";
import { Connection, Transaction } from "@solana/web3.js";
import { findProgramAddress } from "./pda";

const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

export const getTestAccounts = async (start: number, end: number, connection: Connection) => {
    let [mint] = findProgramAddress("sallar");
    let testAccounts: Array<anchor.web3.PublicKey> = [];

    let keyList  = [
        "FdEM5PJpJR5nup9XKwh2prpBXFtGiHi91bS6oHeUUxNQ",
        "E3bWSurPbw3qFL8SZbdrhNe5RTrmqSZkrP6c4NW9w7T3",
        "DYwUX83cGibE215Aa9x8daqJLief1YoVAHrGRG1eewDt",
        "5ghii7qMjPLBa86gqmufEqzeSbXt6WUh8CCrPtdR7vG7",
        "4bowMZ5JU1Zx8wYsz1VgJvkPaBY433JMDEgGiy88Ww5u",
        "J4YARSkhvof5SnxDh9ABDpofTRzb9rtowEab1yh5CyTA",
        "55oRySY4RcJZxyZ77mx3bVeoiyi751M66NY8dC6fT5Uy",
        "GERd8RGXBWFJVZo59g53wotn3H2QjJRxm1FE93ghJREz",
        "GBcXt9Z2XrfoNAhNnuXFZ8nKgVKov3ywgfuvwata9qrc",
        "CD6XkqrvfTbxQSwLgybSEr9ExDJo7VNSA3cS18pVQp4L",
        "4dEP5ZNffc2s8zNbZuiHjafRJ9Wkd86dbj9rFs5yzWT8",
        "AzQY2HkLb22yWReoCfKQNyNo3tUCVQs9Nctb5Eu5WL6S",
        "3zPMvhGwHjJVZ5XTdJNvJmAnriJepdLUPFLUpJhmdNWi",
        "2LTcwkYWCZWxaAMdujAVWtDVnczCUF2QqNbCsXQ7mP1P",
        "C9zVaExTvC1Agv7CmzAR6tpvkr2dor7tKvejLMPPL6U6",
        "HzJPzcR6YuDoLrmqn3mjdQDj4WNdrC4wAvkMB6UkSzTF",
        "5N5Joiwc3r2crhDj26ZfTwG96qdKE1gsuTTx5SeZVmg8",
        "9GL8ZNTQNwuet79BCPmzB4cvoQmQGqCXTMbnNGVfUEe3",
        "9N7t41fXM7UL4YCLuC2rC7ka9aKJd1oRYxYUjmtucinx",
        "6eXSKQ7eKEU1ijyCSEke3wyp8sqn3Q6LLyJP1zsg3WuP",
        "7zpedVQ9Nd3CBAG77qK3wTiMDuAQ26DDTcwC7RnjYzJF",
        "DprCosEU6mDDFDNT84aYbWFxw2dwEqPwyrqxBpcRctPE",
        "9PntDgjxScUSothiwWnnygHdueX1xhK3K6ndkJsR9rTe",
        "GcpQjocq65T9SuEhQWWMwUaGag4SeQCcgDZeGfdJUMgP",
        "2JfiHwdis6nrTobFBCottSNGpuFoiD57vGqdMXzdPEM8"
    ]

    for (let i = start; i < end; i++) {
        testAccounts.push(await getOrCreateAssociatedTokenAccount(provider, mint, new anchor.web3.PublicKey(keyList[i]), connection));
    }

    return testAccounts;
}

export const getOrCreateAssociatedTokenAccount = async (
    provider: anchor.AnchorProvider,
    mint: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey,
    connection: Connection,
): Promise<anchor.web3.PublicKey> => {
    const associatedTokenAddress = await getAssociatedTokenAddress(mint, owner);

    let info = await connection.getAccountInfo(associatedTokenAddress);

    if (info == null) {
        let createAssociatedTokenAccountInstruction_ =
            createAssociatedTokenAccountInstruction(
                provider.wallet.publicKey,
                associatedTokenAddress,
                owner,
                mint,
            );

        const tx = new Transaction();
        tx.add(createAssociatedTokenAccountInstruction_);
        await provider.sendAndConfirm(tx, []);
    }

    return associatedTokenAddress;
};