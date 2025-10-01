import * as d3 from 'd3';
import { show, hide } from '../util/ui-util';
import {
    Delegate,
    DelegateSimilarity,
    DelegateType,
    DelegateVoteCount,
    getVoteValue,
    Network,
    Referendum,
    ReferendumStatus,
    Track,
    VoteCall,
} from '../data/types';
import { Constants } from '../util/constants';
import { COHORT_NUMBERS } from '../data/data-store';
import Headroom from 'headroom.js';

interface UIDelegate {
    onCohortSelectChanged(value: number): void;
    onNetworkSelectChanged(value: string): void;
    onTrackSelectChanged(value: string): void;
    onStatusSelectChanged(value: string): void;
    onDelegateTypeSelectChanged(value: string): void;
    onVotesDownloadButtonClicked(): void;
}

class UI {
    private readonly root: HTMLDivElement;
    private readonly headerContainer: HTMLDivElement;
    private readonly contentContainer: HTMLDivElement;
    private readonly content: HTMLDivElement;
    private readonly title: HTMLDivElement;
    private readonly subtitle: HTMLParagraphElement;
    private readonly loadingContainer: HTMLDivElement;
    private readonly loadingDescription: HTMLDivElement;

    private readonly filterContainer: HTMLDivElement;
    private readonly cohortSelect: HTMLSelectElement;
    private readonly networkSelect: HTMLSelectElement;
    private readonly trackSelect: HTMLSelectElement;
    private readonly statusSelect: HTMLSelectElement;
    private readonly delegateTypeSelect: HTMLSelectElement;
    private readonly delegateTypeFilterItem: HTMLDivElement;

    private readonly voteListDelegateColumn: HTMLDivElement;
    private readonly voteList: HTMLDivElement;
    private readonly votesDownloadButton: HTMLElement;

    private delegate: UIDelegate;

    private similarityGroup: d3.Selection<SVGGElement, unknown, HTMLElement, any> | null = null;
    private responseTimeGroup?: d3.Selection<SVGGElement, unknown, HTMLElement, any>;

    constructor(delegate: UIDelegate) {
        this.delegate = delegate;
        this.root = <HTMLDivElement>document.getElementById('root');
        this.headerContainer = <HTMLDivElement>document.getElementById('page-header-container');
        this.contentContainer = <HTMLDivElement>document.getElementById('content-container');
        this.content = <HTMLDivElement>document.getElementById('content');
        this.title = <HTMLDivElement>document.getElementById('title');
        this.subtitle = <HTMLDivElement>document.getElementById('subtitle');
        this.loadingContainer = <HTMLDivElement>document.getElementById('loading-container');
        this.loadingDescription = <HTMLDivElement>document.getElementById('loading-description');

        this.filterContainer = <HTMLDivElement>document.getElementById('filter-container');
        this.cohortSelect = <HTMLSelectElement>document.getElementById('cohort-select');
        this.networkSelect = <HTMLSelectElement>document.getElementById('network-select');
        this.trackSelect = <HTMLSelectElement>document.getElementById('track-select');
        this.statusSelect = <HTMLSelectElement>document.getElementById('status-select');
        this.delegateTypeSelect = <HTMLSelectElement>(
            document.getElementById('delegate-type-select')
        );
        this.delegateTypeFilterItem = <HTMLDivElement>(
            document.getElementById('delegate-type-filter-item')
        );

        this.voteListDelegateColumn = <HTMLDivElement>(
            document.getElementById('vote-list-delegate-column')
        );
        this.voteList = <HTMLDivElement>document.getElementById('vote-list');
        this.votesDownloadButton = <HTMLElement>document.getElementById('votes-download-button');
        this.votesDownloadButton.addEventListener('click', () => {
            this.delegate.onVotesDownloadButtonClicked();
        });

        const headerHeadroom = new Headroom(this.headerContainer, {
            scroller: this.contentContainer,
            classes: {
                initial: 'header-headroom',
                pinned: 'header-headroom-pinned',
                unpinned: 'header-headroom-unpinned',
            },
        });
        headerHeadroom.init();

        const filterHeadroom = new Headroom(this.filterContainer, {
            scroller: this.contentContainer,
            classes: {
                initial: 'filter-headroom',
                pinned: 'filter-headroom-pinned',
                unpinned: 'filter-headroom-unpinned',
            },
        });
        filterHeadroom.init();
    }

    cleanup() {
        d3.selectAll('*').interrupt();

        d3.select('#vote-count-chart').selectAll('*').remove();
        d3.select('#policy-direction-chart').selectAll('*').remove();
        d3.select('#similarity-matrix-chart').selectAll('*').remove();
        d3.select('#first-vote-time-chart').selectAll('*').remove();
        d3.select('#missed-vote-count-chart').selectAll('*').remove();
        d3.select('#changed-vote-count-chart').selectAll('*').remove();

        // Clear vote list
        this.voteList.innerHTML = '';
        this.voteListDelegateColumn.innerHTML = '';

        // Reset stored references
        this.similarityGroup = null;
        this.responseTimeGroup = undefined;
    }

    lock() {
        hide(this.filterContainer);
        hide(this.content);
        show(this.loadingContainer);
    }

    unlock() {
        show(this.filterContainer);
        show(this.content);
        hide(this.loadingContainer);
    }

    setLoadingDescription(description: string) {
        this.loadingDescription.innerHTML = description;
    }

    setTitleAndSubtitle(cohortNumber: number) {
        this.title.innerHTML = `W3F DV Cohort ${cohortNumber == 4 ? 'IV' : 'V'} Report`;
        let subtitle =
            cohortNumber == 4
                ? 'The fourth cohort of the Decentralized Voices program by Web3 Foundation was <a href="https://medium.com/web3foundation/decentralized-voices-cohort-4-delegates-announced-a5a9c64927fd" target="_blank">announced</a> on the 27th of March, 2025, and the on-chain delegations were dispatched on the 14th of April, 2025. The <span class="bold">delegates are Permanence DAO, The Kus DAO, PolkaWorld, Trustless Core, JAM Implementers DAO (JID), and Polkadot Hungary DAO</span>. The delegates are represented by their short names on the charts for convenience. Green represents <span class="vote-legend aye">aye</span> votes, gray represents <span class="vote-legend abstain">abstain</span>, and red represents <span class="vote-legend nay">nay</span>.'
                : 'The fifth cohort of the Decentralized Voices program by Web3 Foundation was <a href="https://medium.com/web3foundation/decentralized-voices-cohort-5-announced-45fbf1c017ad" target="_blank">announced</a> on the 19th of August, 2025, and the on-chain delegations were dispatched on the 1st of September, 2025. The <span class="bold">delegates are Polkadot Poland DAO, Reeeeeeeeee DAO, PBA Alumni Voting DAO, Saxemberg, Permanence DAO, Trustless Core, Le Nexus, Flez, Cybergov, Daniel Olano, GoverNoun AI, and The White Rabbit</span>. The delegates are represented by their short names on the charts for convenience. Green represents <span class="vote-legend aye">aye</span> votes, gray represents <span class="vote-legend abstain">abstain</span>, and red represents <span class="vote-legend nay">nay</span>.';
        subtitle +=
            '&nbsp;This application is powered by <strong><a href="https://submerge.io" target="_blank">Submerge</a></strong>, <strong><a href="https://github.com/paritytech/subxt" target="_blank">SubXT</a></strong>, and <strong><a href="https://subsquare.io/" target="_blank">Subsquare</a></strong> API.';
        this.subtitle.innerHTML = subtitle;
    }

    initFilters(
        selectedCohortNumber: number,
        networks: Network[],
        tracks: Track[],
        statuses: ReferendumStatus[],
        delegateTypes: DelegateType[],
    ) {
        let cohortSelectHTML = '';
        for (const cohortNumber of COHORT_NUMBERS) {
            cohortSelectHTML += `<option value="${cohortNumber}" ${cohortNumber == selectedCohortNumber ? 'selected' : ''}>Cohort ${cohortNumber}</option>`;
        }
        this.cohortSelect.innerHTML = cohortSelectHTML;
        this.cohortSelect.onchange = (_) => {
            const cohortNumber = Number.parseInt(this.cohortSelect.value);
            if (cohortNumber == 4) {
                hide(this.delegateTypeFilterItem);
            } else {
                show(this.delegateTypeFilterItem);
            }
            this.delegate.onCohortSelectChanged(cohortNumber);
        };

        let networkSelectHTML = '<option value="all" selected>All Networks</option>';
        networks.forEach((n) => {
            networkSelectHTML += `<option value="${n.id}">${n.display}</option>`;
        });
        this.networkSelect.innerHTML = networkSelectHTML;
        this.networkSelect.onchange = (_) => {
            this.delegate.onNetworkSelectChanged(this.networkSelect.value);
        };

        let trackSelectHTML = '<option value="all">All Tracks</option>';
        trackSelectHTML += '<option value="dv" selected>DV Tracks</option>';
        for (const track of tracks) {
            trackSelectHTML += `<option value="${track.id}">${track.name}</option>`;
        }
        this.trackSelect.innerHTML = trackSelectHTML;
        this.trackSelect.onchange = (_) => {
            this.delegate.onTrackSelectChanged(this.trackSelect.value);
        };

        let statusSelectHTML = '<option value="all">All Statuses</option>';
        for (const status of statuses) {
            statusSelectHTML += `<option value="${status.id}">${status.status}</option>`;
        }
        this.statusSelect.innerHTML = statusSelectHTML;
        this.statusSelect.onchange = (_) => {
            this.delegate.onStatusSelectChanged(this.statusSelect.value);
        };

        let delegateTypeSelectHTML = '<option value="all">All DVs</option>';
        for (const delegateType of delegateTypes) {
            delegateTypeSelectHTML += `<option value="${delegateType.id}">${delegateType.name}</option>`;
        }
        this.delegateTypeSelect.innerHTML = delegateTypeSelectHTML;
        this.delegateTypeSelect.onchange = (_) => {
            this.delegate.onDelegateTypeSelectChanged(this.delegateTypeSelect.value);
        };
    }

    displayVoteCountChart(data: DelegateVoteCount[]) {
        type StackedDatum = d3.SeriesPoint<DelegateVoteCount> & { key: keyof DelegateVoteCount };
        const totals = data.map((d) => ({
            delegateId: d.delegateId,
            delegateName: d.delegateName,
            delegateShortName: d.delegateShortName,
            total: d.nayCount + d.abstainCount + d.ayeCount,
        }));
        const stackKeys = ['nayCount', 'abstainCount', 'ayeCount'] as const;
        const color = d3
            .scaleOrdinal<string>()
            .domain(stackKeys)
            .range([Constants.NAY_COLOR, Constants.ABSTAIN_COLOR, Constants.AYE_COLOR]);

        const width = 800;
        const height = 45 * data.length;
        const margin = { top: 12, right: 20, bottom: 16, left: 80 };

        const svg = d3
            .select<SVGSVGElement, unknown>('#vote-count-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);

        // Fixed max x-domain for smooth updates
        const max = d3.max(data, (d) => stackKeys.reduce((sum, key) => sum + d[key], 0))!;
        // this.voteCountsMaxX = Math.max(this.voteCountsMaxX, newMax);

        const x = d3
            .scaleLinear()
            .domain([0, max + Math.floor(max / 10)])
            .range([margin.left, width - margin.right]);
        const y = d3
            .scaleBand()
            .domain(data.map((d) => d.delegateShortName))
            .range([margin.top, height - margin.bottom])
            .padding(0.1);
        const stackedData = d3.stack<DelegateVoteCount>().keys(stackKeys)(data);
        // bars
        const barGroups = svg
            .selectAll<SVGGElement, d3.Series<DelegateVoteCount, string>>('g.layer')
            .data(stackedData, (d: any) => d.key);
        const barGroupsEnter = barGroups
            .enter()
            .append('g')
            .attr('class', 'layer')
            .attr('fill', (d) => color(d.key)!);
        barGroupsEnter
            .merge(barGroups)
            .selectAll<SVGRectElement, StackedDatum>('rect')
            .data(
                (d) =>
                    d.map((point) =>
                        Object.assign(point, { key: d.key as keyof DelegateVoteCount }),
                    ),
                (d) => d.data.delegateId + '-' + d.key,
            )
            .join(
                (enter) =>
                    enter
                        .append('rect')
                        .attr('x', (d) => x(d[0]))
                        .attr('y', (d) => y(d.data.delegateShortName)!)
                        .attr('width', (d) => x(d[1]) - x(d[0]))
                        .attr('height', y.bandwidth()),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => x(d[0]))
                        .attr('width', (d) => x(d[1]) - x(d[0]))
                        .attr('y', (d) => y(d.data.delegateShortName)!)
                        .attr('height', y.bandwidth()),
                (exit) => exit.remove(),
            );
        // labels
        barGroupsEnter
            .merge(barGroups)
            .selectAll<SVGTextElement, StackedDatum>('text')
            .data(
                (d) =>
                    d.map((point) =>
                        Object.assign(point, { key: d.key as keyof DelegateVoteCount }),
                    ),
                (d) => d.data.delegateId + '-' + d.key,
            )
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('text-anchor', 'middle')
                        .attr('dy', '0.35em')
                        .style('fill', 'white')
                        .style('font-size', '10px')
                        .attr('x', (d) => {
                            return x(d[0]) + (x(d[1]) - x(d[0])) / 2;
                        })
                        .attr('y', (d) => y(d.data.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => {
                            const w = x(d[1]) - x(d[0]);
                            return w > 10 ? String(d.data[d.key]) : '';
                        }),

                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => {
                            return x(d[0]) + (x(d[1]) - x(d[0])) / 2;
                        })
                        .attr('y', (d) => y(d.data.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => {
                            const w = x(d[1]) - x(d[0]);
                            return w > 10 ? String(d.data[d.key]) : '';
                        }),

                (exit) => exit.remove(),
            );
        // total labels at the end of each stacked bar
        svg.selectAll<SVGTextElement, (typeof totals)[0]>('.total-label')
            .data(totals, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'total-label')
                        .attr('x', (d) => x(d.total) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'start')
                        .style('fill', 'black')
                        .style('font-size', '10px')
                        .text((d) => d.total),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => x(d.total) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => d.total),
                (exit) => exit.remove(),
            );
        // axes
        svg.selectAll('.x-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'x-axis')
                        .attr('transform', `translate(0,${height - margin.bottom})`)
                        .call(d3.axisBottom(x)),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        // @ts-ignore
                        .call(d3.axisBottom(x)),
            );
        svg.selectAll('.y-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'y-axis')
                        .attr('transform', `translate(${margin.left},0)`)
                        .call((g) => {
                            g.call(d3.axisLeft(y));
                            g.selectAll('text')
                                .style('font-size', '11px')
                                .style('font-family', 'Inter');
                        }),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .call((g) => {
                            // @ts-ignore
                            g.call(d3.axisLeft(y));
                            g.selectAll('text')
                                .style('font-size', '11px')
                                .style('font-family', 'Inter');
                        }),
            );
        // cleanup exit
        barGroups.exit().remove();
    }

    displayFeedbackRateChart(data: DelegateVoteCount[]) {
        const width = 800;
        const height = 42 * data.length;
        const margin = { top: 12, right: 20, bottom: 20, left: 80 };

        const svg = d3
            .select<SVGSVGElement, unknown>('#feedback-rate-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);
        svg.selectAll('*').remove();

        const sortedData = [...data].sort((a, b) => {
            const aTotal = a.ayeCount + a.nayCount + a.abstainCount;
            const aRate = aTotal > 0 ? a.feedbackCount / aTotal : 0;
            const bTotal = b.ayeCount + b.nayCount + b.abstainCount;
            const bRate = bTotal > 0 ? b.feedbackCount / bTotal : 0;
            return bRate - aRate;
        });
        const maxFeedbackRate = d3.max(sortedData, (d) => {
            const total = d.ayeCount + d.nayCount + d.abstainCount;
            return total > 0 ? (d.feedbackCount / total) * 100 : 0;
        })!;
        const x = d3
            .scaleLinear()
            .domain([0, maxFeedbackRate + 5])
            .range([margin.left, width - margin.right]);

        const y = d3
            .scaleBand()
            .domain(sortedData.map((d) => d.delegateShortName))
            .range([margin.top, height - margin.bottom])
            .padding(0.1);

        const getBarColor = (feedbackCount: number, totalCount: number): string => {
            if (totalCount === 0) return '#f44336'; // Red for 0%
            const rate = totalCount > 0 ? feedbackCount / totalCount : 0;

            if (rate <= 0.5) {
                // Interpolate from red to gray (0% to 50%)
                return d3.interpolate('#f44336', '#aaaaaa')(rate * 2);
            } else {
                // Interpolate from gray to green (50% to 100%)
                return d3.interpolate('#aaaaaa', '#4caf50')((rate - 0.5) * 2);
            }
        };

        // bars
        svg.selectAll<SVGRectElement, DelegateVoteCount>('.feedback-rate-bar')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('rect')
                        .attr('class', 'feedback-rate-bar')
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return x(rate) - x(0);
                        })
                        .attr('height', y.bandwidth())
                        .attr('fill', (d) =>
                            getBarColor(d.feedbackCount, d.ayeCount + d.nayCount + d.abstainCount),
                        ),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return x(rate) - x(0);
                        })
                        .attr('fill', (d) =>
                            getBarColor(d.feedbackCount, d.ayeCount + d.nayCount + d.abstainCount),
                        ),
                (exit) => exit.remove(),
            );
        // labels at the end of the bars
        svg.selectAll<SVGTextElement, DelegateVoteCount>('.feedback-rate-label')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'feedback-rate-label')
                        .attr('x', (d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return x(rate) + 4;
                        })
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'start')
                        .style('fill', 'black')
                        .style('font-size', '10px')
                        .text((d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return `${Math.floor(rate)}%`;
                        }),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return x(rate) + 4;
                        })
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => {
                            const total = d.ayeCount + d.nayCount + d.abstainCount;
                            const rate = total > 0 ? (d.feedbackCount / total) * 100 : 0;
                            return `${Math.floor(rate)}%`;
                        }),
                (exit) => exit.remove(),
            );
        // x axis
        svg.selectAll('.x-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'x-axis')
                        .attr('transform', `translate(0,${height - margin.bottom})`)
                        .call(
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .call(
                            // @ts-ignore
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
            );
        // y axis
        svg.selectAll('.y-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'y-axis')
                        .attr('transform', `translate(${margin.left},0)`)
                        .call((g) => {
                            g.call(d3.axisLeft(y));
                            g.selectAll('text')
                                .style('font-size', '11px')
                                .style('font-family', 'Inter');
                        }),
                (update) =>
                    update.attr('transform', `translate(${margin.left},0)`).call((g) => {
                        // @ts-ignore
                        g.call(d3.axisLeft(y));
                        g.selectAll('text')
                            .style('font-size', '11px')
                            .style('font-family', 'Inter');
                    }),
            );
    }

    displayPolicyDirectionChart(data: DelegateVoteCount[]) {
        const width = 800;
        const height = 40 * data.length;
        const margin = { top: 12, right: 20, bottom: 20, left: 80 };

        const svg = d3
            .select<SVGSVGElement, unknown>('#policy-direction-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);
        svg.selectAll('*').remove();

        const scoredData = data.map((d) => ({
            ...d,
            score: d.ayeCount - d.nayCount,
        }));
        scoredData.sort((a, b) => b.score - a.score);

        const maxScore = d3.max(scoredData, (d) => Math.abs(d.score))!;

        const x = d3
            .scaleLinear()
            .domain([-maxScore, maxScore])
            .range([margin.left, width - margin.right]);

        const y = d3
            .scaleBand()
            .domain(scoredData.map((d) => d.delegateShortName))
            .range([margin.top, height - margin.bottom])
            .padding(0.1);

        const xAxis = svg.selectAll<SVGGElement, unknown>('.x-axis').data([null]);
        xAxis.join(
            (enter) =>
                enter
                    .append('g')
                    .attr('class', 'x-axis')
                    .attr('transform', `translate(0,${height - margin.bottom})`)
                    .call(d3.axisBottom(x).ticks(5)),
            (update) =>
                update
                    .transition()
                    .duration(Constants.CHART_TRANSITION_TIME_MS)
                    .call(d3.axisBottom(x).ticks(5)),
        );

        const yAxis = svg.selectAll<SVGGElement, unknown>('.y-axis').data([null]);
        yAxis.join(
            (enter) =>
                enter
                    .append('g')
                    .attr('class', 'y-axis')
                    .attr('transform', `translate(${margin.left},0)`)
                    .call((g) =>
                        g
                            .call(d3.axisLeft(y))
                            .selectAll('text')
                            .style('font-size', '11px')
                            .style('font-family', 'Inter'),
                    ),
            (update) =>
                update
                    .transition()
                    .duration(Constants.CHART_TRANSITION_TIME_MS)
                    .call((g) =>
                        g
                            .call(d3.axisLeft(y))
                            .selectAll('text')
                            .style('font-size', '11px')
                            .style('font-family', 'Inter'),
                    ),
        );

        // bars
        const bars = svg
            .selectAll<SVGRectElement, (typeof scoredData)[0]>('.bar')
            .data(scoredData, (d) => d.delegateId);
        bars.join(
            (enter) =>
                enter
                    .append('rect')
                    .attr('class', 'bar')
                    .attr('x', (d) => (d.score === 0 ? x(0) - 1 : x(Math.min(0, d.score))))
                    .attr('y', (d) => y(d.delegateShortName)!)
                    .attr('width', (d) => (d.score === 0 ? 2 : Math.abs(x(d.score) - x(0))))
                    .attr('height', y.bandwidth())
                    .attr('fill', (d) =>
                        d.score > 0
                            ? Constants.AYE_COLOR
                            : d.score < 0
                              ? Constants.NAY_COLOR
                              : Constants.ABSTAIN_COLOR,
                    ),

            (update) =>
                update
                    .transition()
                    .duration(Constants.CHART_TRANSITION_TIME_MS)
                    .attr('x', (d) => (d.score === 0 ? x(0) - 1 : x(Math.min(0, d.score))))
                    .attr('width', (d) => (d.score === 0 ? 2 : Math.abs(x(d.score) - x(0))))
                    .attr('y', (d) => y(d.delegateShortName)!)
                    .attr('height', y.bandwidth())
                    .attr('fill', (d) =>
                        d.score > 0
                            ? Constants.AYE_COLOR
                            : d.score < 0
                              ? Constants.NAY_COLOR
                              : Constants.ABSTAIN_COLOR,
                    ),

            (exit) => exit.remove(),
        );

        const labels = svg
            .selectAll<SVGTextElement, (typeof scoredData)[0]>('.bar-label')
            .data(scoredData, (d) => d.delegateId);
        // enter selection (no transition)
        labels
            .enter()
            .append('text')
            .attr('class', 'bar-label')
            .style('fill', 'white')
            .style('font-size', '11px')
            .attr('text-anchor', 'middle')
            .attr('dy', '0.35em')
            .attr('x', (d) => {
                const barStart = x(Math.min(0, d.score));
                const barEnd = x(Math.max(0, d.score));
                const barWidth = Math.abs(barEnd - barStart);
                return barStart + barWidth / 2;
            })
            .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
            .text((d) => {
                const barStart = x(Math.min(0, d.score));
                const barEnd = x(Math.max(0, d.score));
                const barWidth = Math.abs(barEnd - barStart);
                const plusSign = d.score > 0 ? '+' : '';
                return barWidth > 10 ? `${plusSign}${d.score}` : '';
            });
        // update selection (with transition)
        labels
            .transition()
            .duration(Constants.CHART_TRANSITION_TIME_MS)
            .attr('x', (d) => {
                const barStart = x(Math.min(0, d.score));
                const barEnd = x(Math.max(0, d.score));
                const barWidth = Math.abs(barEnd - barStart);
                return barStart + barWidth / 2;
            })
            .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
            .text((d) => {
                const barStart = x(Math.min(0, d.score));
                const barEnd = x(Math.max(0, d.score));
                const barWidth = Math.abs(barEnd - barStart);
                const plusSign = d.score > 0 ? '+' : '';
                return barWidth > 10 ? `${plusSign}${d.score}` : '';
            });
    }

    displaySimilarityMatrixChart(delegates: Delegate[], similarities: DelegateSimilarity[]) {
        const cellWidth = Math.floor(672 / delegates.length);
        const cellHeight = Math.floor(300 / delegates.length);
        const margin = { top: 50, left: 70, bottom: 20, right: 20 };
        const width = (delegates.length - 1) * cellWidth + margin.left + margin.right;
        const height = (delegates.length - 1) * cellHeight + margin.top + margin.bottom;

        const svg = d3
            .select<SVGSVGElement, unknown>('#similarity-matrix-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);
        svg.selectAll('*').remove();

        const color = d3
            .scaleLinear<string>()
            .domain([-1, 0, 1])
            .range([Constants.NAY_COLOR, Constants.ABSTAIN_COLOR, Constants.AYE_COLOR]);

        const radius = d3
            .scaleSqrt()
            .domain([0, 1])
            .range([0, Math.min(cellWidth, cellHeight) / 2 - 1]);

        if (svg.select('.grid-lines').empty()) {
            // Hhrizontal grid lines
            svg.append('g')
                .attr('class', 'grid-lines horizontal')
                .attr('stroke', '#dddddd')
                .attr('stroke-width', 0.5)
                .selectAll('line')
                .data(d3.range(delegates.length - 1))
                .join('line')
                .attr('x1', margin.left)
                .attr('x2', margin.left + (delegates.length - 1) * cellWidth)
                .attr('y1', (d) => margin.top + d * cellHeight + cellHeight / 2)
                .attr('y2', (d) => margin.top + d * cellHeight + cellHeight / 2);

            // vertical grid lines
            svg.append('g')
                .attr('class', 'grid-lines vertical')
                .attr('stroke', '#dddddd')
                .attr('stroke-width', 0.5)
                .selectAll('line')
                .data(d3.range(delegates.length - 1))
                .join('line')
                .attr('y1', margin.top)
                .attr('y2', margin.top + (delegates.length - 1) * cellHeight)
                .attr('x1', (d) => margin.left + d * cellWidth + cellWidth / 2)
                .attr('x2', (d) => margin.left + d * cellWidth + cellWidth / 2);

            // row labels
            svg.append('g')
                .selectAll('text.row-label')
                .data(delegates.slice(0, delegates.length - 1))
                .join('text')
                .attr('class', 'row-label')
                .attr('x', margin.left - 10)
                .attr('y', (_, i) => margin.top + i * cellHeight + cellHeight / 2)
                .attr('dy', '0.35em')
                .attr('text-anchor', 'end')
                .style('font-size', '9px')
                .style('font-family', 'Inter')
                .text((d) => d.shortName);

            // column labels
            svg.append('g')
                .selectAll('text.column-label')
                .data(delegates.slice(1, delegates.length))
                .join('text')
                .attr('class', 'column-label')
                .attr(
                    'x',
                    (_, i) => margin.left + (delegates.length - 2 - i) * cellWidth + cellWidth / 2,
                )
                .attr('y', margin.top - 12)
                .attr('text-anchor', 'middle')
                .style('font-size', '9px')
                .style('font-family', 'Inter')
                .text((d) => d.shortName);

            // Create the similarity group container
            this.similarityGroup = svg
                .append('g')
                .attr('class', 'similarity-group')
                .attr('transform', `translate(${margin.left},${margin.top})`);
        }

        // Compute similarity pairs
        const pairs: { a: Delegate; b: Delegate; value: number; row: number; col: number }[] = [];
        for (let i = 0; i < delegates.length; i++) {
            for (let j = i; j < delegates.length; j++) {
                if (i === j) continue;
                const a = delegates[i];
                const b = delegates[j];
                const similarity = similarities.find(
                    (s) => (s.aId == a.id && s.bId == b.id) || (s.aId == b.id && s.bId == a.id),
                )!;
                pairs.push({ a, b, value: similarity.value, row: i, col: j });
            }
        }

        // similarity circles
        this.similarityGroup!.selectAll<SVGCircleElement, (typeof pairs)[0]>('circle')
            .data(pairs, (d) => `${d.a.id}-${d.b.id}`)
            .join(
                (enter) =>
                    enter
                        .append('circle')
                        .attr(
                            'cx',
                            (d) => (delegates.length - 1 - d.col) * cellWidth + cellWidth / 2,
                        )
                        .attr('cy', (d) => d.row * cellHeight + cellHeight / 2)
                        .attr('r', (d) => radius(Math.abs(d.value)))
                        .attr('fill', (d) => color(d.value)),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr(
                            'cx',
                            (d) => (delegates.length - 1 - d.col) * cellWidth + cellWidth / 2,
                        )
                        .attr('cy', (d) => d.row * cellHeight + cellHeight / 2)
                        .attr('r', (d) => radius(Math.abs(d.value)))
                        .attr('fill', (d) => color(d.value)),
                (exit) => exit.remove(),
            );
    }

    private blocksToTime(blocks: number): string {
        const totalSeconds = blocks * 6;
        const days = Math.floor(totalSeconds / 86400);
        const hours = Math.floor((totalSeconds % 86400) / 3600);
        const minutes = Math.floor((totalSeconds % 3600) / 60);
        const seconds = totalSeconds % 60;

        const parts = [];
        if (days > 0) parts.push(`${days}d`);
        if (hours > 0) parts.push(`${hours}h`);
        if (minutes > 0) parts.push(`${minutes}m`);
        if (parts.length === 0 || (days === 0 && hours === 0)) {
            parts.push(`${seconds}s`);
        }

        return parts.join(' ');
    }

    displayFirstVoteTimeChart(responseTimeMap: Map<Delegate, number>) {
        const delegatesWithTimes = Array.from(responseTimeMap.entries())
            .map(([delegate, blocks]) => ({ delegate, blocks }))
            .sort((a, b) => a.blocks - b.blocks); // Fastest first

        if (delegatesWithTimes.length === 0) {
            console.warn('No data to display in response time chart.');
            return;
        }

        const margin = { top: 12, right: 30, bottom: 32, left: 80 };
        const barHeight = 38;
        const width = 700;
        const height = delegatesWithTimes.length * barHeight + margin.top + margin.bottom;

        const svg = d3
            .select<SVGSVGElement, unknown>('#first-vote-time-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);

        const computedMaxTime = d3.max(delegatesWithTimes, (d) => d.blocks)! + 10 * 60 * 12;
        const x = d3
            .scaleLinear()
            .domain([0, computedMaxTime])
            .range([0, width - margin.left - margin.right]);

        // x-axis
        if (svg.select('.x-axis').empty()) {
            const xAxisGroup = svg
                .append('g')
                .attr('class', 'x-axis')
                .attr('transform', `translate(${margin.left},${height - margin.bottom})`)
                .call(d3.axisBottom(x).ticks(5));

            xAxisGroup
                .append('text')
                .attr('class', 'x-axis-label')
                .attr('x', (x.range()[0] + x.range()[1]) / 2)
                .attr('y', 28)
                .attr('fill', 'black')
                .attr('text-anchor', 'end')
                .style('font-size', '8px')
                .style('font-family', 'Inter')
                .text('blocks');
        } else {
            svg.select<SVGGElement>('.x-axis')
                .transition()
                .duration(Constants.CHART_TRANSITION_TIME_MS)
                .call(d3.axisBottom(x).ticks(5));
        }

        // group container
        if (!this.responseTimeGroup) {
            this.responseTimeGroup = svg
                .append('g')
                .attr('class', 'response-time-group')
                .attr('transform', `translate(${margin.left}, ${margin.top})`);
        }

        const group = this.responseTimeGroup;

        // bars
        group
            .selectAll<SVGRectElement, (typeof delegatesWithTimes)[0]>('rect')
            .data(delegatesWithTimes, (d) => d.delegate.id)
            .join(
                (enter) =>
                    enter
                        .append('rect')
                        .attr('x', 0)
                        .attr('y', (_, i) => i * barHeight)
                        .attr('width', (d) => x(d.blocks ?? 0))
                        .attr('height', barHeight - 6)
                        .attr('fill', '#3b82f6'),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('width', (d) => x(d.blocks ?? 0))
                        .attr('y', (_, i) => i * barHeight),
            );

        // bar labels (value inside bars)
        group
            .selectAll<SVGTextElement, (typeof delegatesWithTimes)[0]>('text.value')
            .data(delegatesWithTimes, (d) => d.delegate.id)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'value')
                        .attr('x', (d) => x(d.blocks ?? 0) / 2)
                        .attr('y', (_, i) => i * barHeight + (barHeight - 6) / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'middle')
                        .style('fill', 'white')
                        .style('font-size', '9.2px')
                        .style('font-family', 'Inter')
                        .text((d) => this.blocksToTime(d.blocks ?? 0)),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => x(d.blocks ?? 0) / 2)
                        .attr('y', (_, i) => i * barHeight + (barHeight - 6) / 2)
                        .text((d) => this.blocksToTime(d.blocks ?? 0)),
            );

        // left-side labels
        group
            .selectAll<SVGTextElement, (typeof delegatesWithTimes)[0]>('text.label')
            .data(delegatesWithTimes, (d) => d.delegate.id)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'label')
                        .attr('x', -10)
                        .attr('y', (_, i) => i * barHeight + (barHeight - 6) / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'end')
                        .style('font-size', '10px')
                        .style('font-family', 'Inter')
                        .text((d) => d.delegate.shortName),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('y', (_, i) => i * barHeight + (barHeight - 6) / 2)
                        .text((d) => d.delegate.shortName),
            );
    }

    displayMissedVoteCountChart(data: DelegateVoteCount[]) {
        const width = 800;
        const height = 40 * data.length;
        const margin = { top: 12, right: 20, bottom: 20, left: 80 };

        const svg = d3
            .select<SVGSVGElement, unknown>('#missed-vote-count-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);

        const sortedData = [...data].sort((a, b) => a.missedCount - b.missedCount);
        const maxMissed = d3.max(sortedData, (d) => d.missedCount)!;
        const x = d3
            .scaleLinear()
            .domain([0, maxMissed + 5])
            .range([margin.left, width - margin.right]);

        const y = d3
            .scaleBand()
            .domain(sortedData.map((d) => d.delegateShortName))
            .range([margin.top, height - margin.bottom])
            .padding(0.1);

        // bars
        svg.selectAll<SVGRectElement, DelegateVoteCount>('.missed-bar')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('rect')
                        .attr('class', 'missed-bar')
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => x(d.missedCount) - x(0))
                        .attr('height', y.bandwidth())
                        .attr('fill', '#aaaaaa'),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => x(d.missedCount) - x(0))
                        .attr('height', y.bandwidth()),
                (exit) => exit.remove(),
            );
        // labels at the end of the bars
        svg.selectAll<SVGTextElement, DelegateVoteCount>('.missed-label')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'missed-label')
                        .attr('x', (d) => x(d.missedCount) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'start')
                        .style('fill', 'black')
                        .style('font-size', '10px')
                        .text((d) => d.missedCount),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => x(d.missedCount) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => d.missedCount),
                (exit) => exit.remove(),
            );
        // x axis
        svg.selectAll('.x-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'x-axis')
                        .attr('transform', `translate(0,${height - margin.bottom})`)
                        .call(
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .call(
                            // @ts-ignore
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
            );
        // y axis
        svg.selectAll('.y-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'y-axis')
                        .attr('transform', `translate(${margin.left},0)`)
                        .call((g) => {
                            g.call(d3.axisLeft(y));
                            g.selectAll('text')
                                .style('font-size', '11px')
                                .style('font-family', 'Inter');
                        }),
                (update) =>
                    update.attr('transform', `translate(${margin.left},0)`).call((g) => {
                        // @ts-ignore
                        g.call(d3.axisLeft(y));
                        g.selectAll('text')
                            .style('font-size', '11px')
                            .style('font-family', 'Inter');
                    }),
            );
    }

    displayChangedVoteCountChart(data: DelegateVoteCount[]) {
        const width = 800;
        const height = 40 * data.length;
        const margin = { top: 12, right: 20, bottom: 20, left: 80 };

        const svg = d3
            .select<SVGSVGElement, unknown>('#changed-vote-count-chart')
            .attr('viewBox', `0 0 ${width} ${height}`);

        const sortedData = [...data].sort((a, b) => a.changedCount - b.changedCount);
        const maxMissed = d3.max(sortedData, (d) => d.changedCount)!;
        const x = d3
            .scaleLinear()
            .domain([0, maxMissed + 5])
            .range([margin.left, width - margin.right]);

        const y = d3
            .scaleBand()
            .domain(sortedData.map((d) => d.delegateShortName))
            .range([margin.top, height - margin.bottom])
            .padding(0.1);

        // bars
        svg.selectAll<SVGRectElement, DelegateVoteCount>('.missed-bar')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('rect')
                        .attr('class', 'missed-bar')
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => x(d.changedCount) - x(0))
                        .attr('height', y.bandwidth())
                        .attr('fill', '#aaaaaa'),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', x(0))
                        .attr('y', (d) => y(d.delegateShortName)!)
                        .attr('width', (d) => x(d.changedCount) - x(0))
                        .attr('height', y.bandwidth()),
                (exit) => exit.remove(),
            );
        // labels at the end of the bars
        svg.selectAll<SVGTextElement, DelegateVoteCount>('.missed-label')
            .data(sortedData, (d) => d.delegateId)
            .join(
                (enter) =>
                    enter
                        .append('text')
                        .attr('class', 'missed-label')
                        .attr('x', (d) => x(d.changedCount) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .attr('dy', '0.35em')
                        .attr('text-anchor', 'start')
                        .style('fill', 'black')
                        .style('font-size', '10px')
                        .text((d) => d.changedCount),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .attr('x', (d) => x(d.changedCount) + 4)
                        .attr('y', (d) => y(d.delegateShortName)! + y.bandwidth() / 2)
                        .text((d) => d.changedCount),
                (exit) => exit.remove(),
            );
        // x axis
        svg.selectAll('.x-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'x-axis')
                        .attr('transform', `translate(0,${height - margin.bottom})`)
                        .call(
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
                (update) =>
                    update
                        .transition()
                        .duration(Constants.CHART_TRANSITION_TIME_MS)
                        .call(
                            // @ts-ignore
                            d3
                                .axisBottom(x)
                                .ticks(5)
                                .tickFormat((d) => Math.round(d.valueOf()).toString()),
                        ),
            );
        // y axis
        svg.selectAll('.y-axis')
            .data([null])
            .join(
                (enter) =>
                    enter
                        .append('g')
                        .attr('class', 'y-axis')
                        .attr('transform', `translate(${margin.left},0)`)
                        .call((g) => {
                            g.call(d3.axisLeft(y));
                            g.selectAll('text')
                                .style('font-size', '11px')
                                .style('font-family', 'Inter');
                        }),
                (update) =>
                    update.attr('transform', `translate(${margin.left},0)`).call((g) => {
                        // @ts-ignore
                        g.call(d3.axisLeft(y));
                        g.selectAll('text')
                            .style('font-size', '11px')
                            .style('font-family', 'Inter');
                    }),
            );
    }

    displayVoteList(
        networks: Network[],
        delegates: Delegate[],
        referenda: Referendum[],
        lastVoteMaps: Map<string, Map<string, VoteCall>>,
    ) {
        let delegateColumnHTML = '<div class="item delegate">&nbsp;</div>';
        for (const delegate of delegates) {
            delegateColumnHTML += `<div class="item delegate bold">${delegate.shortName}</div>`;
        }
        this.voteListDelegateColumn.innerHTML = delegateColumnHTML;
        let voteListHTML = '';
        for (const referendum of referenda) {
            const network = networks.find((n) => n.id == referendum.networkId)!;
            const referendumURL = `https://${network.chain}.subsquare.io/referenda/${referendum.index}`;
            const referendumIndexDisplay = `${network.tokenTicker + '&nbsp;'}${referendum.index}`;
            let referendumColumnHTML = `<div class="item bold referendum-index ${referendum.isRetracted ? 'retracted' : ''}"><a href="${referendumURL}" target="_blank">${referendumIndexDisplay}</a></div>`;
            for (const delegate of delegates) {
                const voteMap = lastVoteMaps.get(delegate.id)!;
                const key = `${referendum.networkId}_${referendum.index}`;
                if (voteMap.has(key)) {
                    const voteCall = voteMap.get(key)!;
                    const voteValue = getVoteValue(voteCall);
                    let voteIndicator;
                    if (voteValue > 0) {
                        voteIndicator = `<div class="vote-indicator aye"></div>`;
                    } else if (voteValue == 0) {
                        voteIndicator = `<div class="vote-indicator abstain"></div>`;
                    } else {
                        voteIndicator = `<div class="vote-indicator nay"></div>`;
                    }
                    let feedbackIndicator = '';
                    if (!referendum.isRetracted) {
                        if (
                            voteCall.subsquareCommentId != undefined ||
                            voteCall.polkassemblyCommentId != undefined
                        ) {
                            feedbackIndicator = '<span>💬</span>';
                        } else {
                            feedbackIndicator = '<span>⚠️</span>';
                        }
                    }
                    const extrinsicURL = `https://${network.chain}.subscan.io/extrinsic/0x${voteCall.extrinsicHash}`;
                    const extrinsicDisplay = `${voteCall.block.number}-${voteCall.extrinsicIndex}`;
                    referendumColumnHTML += `<div class="item">${voteIndicator}<a href="${extrinsicURL}" target="_blank">${extrinsicDisplay}</a>${feedbackIndicator}</div>`;
                } else {
                    referendumColumnHTML += `<div class="item">-</div>`;
                }
            }
            voteListHTML += `<div class="referendum-column">${referendumColumnHTML}</div>`;
        }
        this.voteList.innerHTML = voteListHTML;
    }
}

export { UI, UIDelegate };
