interface Block {
    timestamp: number;
    number: number;
    hash: string;
    parentHash: string;
}

interface Track {
    networkId: number;
    id: number;
    name: string;
}

interface Network {
    id: number;
    chain: string;
    display: string;
    rpcUrl: string;
    tokenTicker: string;
    tokenDecimals: number;
    tokenFormatDecimalPoints: number;
    ss58Prefix: number;
    tracks: Track[];
    cohorts: Cohort[];
}

interface Cohort {
    number: number;
    network: Network;
    announcementDate: Date;
    announcementUrl?: string;
    delegationDate: Date;
    startBlock: Block;
    tracks: Track[];
}

interface ReferendumStatus {
    id: number;
    status: string;
}

interface Delegation {
    id: number;
    cohortNumber: number;
    networkId: number;
    delegatorAccountId: string;
    delegateId: string;
    delegateAccountId: string;
    startBlock: Block;
    startExtrinsicHash: string;
    startExtrinsicIndex: number;
    endBlock?: Block;
    endExtrinsicHash?: string;
    endExtrinsicIndex?: number;
}

interface DelegateType {
    id: number;
    name: string;
    code: string;
}

interface Delegate {
    id: string;
    typeId: number;
    name: string;
    shortName: string;
    url?: string;
    twitter?: string;
    delegations: Delegation[];
    votes: VoteCall[];
}

interface Referendum {
    networkId: number;
    index: number;
    track: Track;
    submissionBlock: Block;
    status: ReferendumStatus;
    isRetracted: boolean;
}

interface VoteCall {
    id: number;
    networkId: number;
    referendumIndex: number;
    block: Block;
    extrinsicIndex: number;
    extrinsicHash: string;
    isBatch: boolean;
    isMultisig: boolean;
    isMultisigExecuted: boolean;
    isProxy: boolean;
    isSuccessful: boolean;
    signerAccountId: string;
    voterAccountId: string;
    voteType: string;
    isAye?: boolean;
    conviction?: number;
    balance?: string;
    aye?: string;
    nay?: string;
    abstain?: string;
    subsquareCommentId?: string;
    polkassemblyCommentId?: string;
}

function getVoteValue(vote: VoteCall): number {
    switch (vote.voteType) {
        case 'standard': {
            if (vote.isAye!) {
                return 1;
            } else {
                return -1;
            }
        }
        default: {
            return 0;
        }
    }
}

type DelegateVoteCount = {
    delegateId: string;
    delegateName: string;
    delegateShortName: string;
    nayCount: number;
    abstainCount: number;
    ayeCount: number;
    missedCount: number;
    changedCount: number;
    feedbackCount: number;
};

type DelegateSimilarity = {
    aId: string;
    bId: string;
    value: number;
};

export {
    Block,
    Cohort,
    DelegateType,
    Delegate,
    Network,
    Referendum,
    ReferendumStatus,
    Track,
    VoteCall,
    getVoteValue,
    DelegateVoteCount,
    DelegateSimilarity,
};
