/**
 * Audit Page Object
 *
 * @description Page object for audit trail and hash chain verification
 */

import { BasePage } from './BasePage';

class AuditPage extends BasePage {
  // =============
  // Selectors
  // =============

  get auditView() {
    return $('[data-testid="audit-view"]');
  }

  get auditTimeline() {
    return $('[data-testid="audit-timeline"]');
  }

  get auditEvents() {
    return $$('[data-testid="audit-event"]');
  }

  get chainStatus() {
    return $('[data-testid="chain-status"]');
  }

  get chainStatusIcon() {
    return $('[data-testid="chain-status-icon"]');
  }

  get verifyChainButton() {
    return $('button*=Hash Chain verifizieren');
  }

  get eventCount() {
    return $('[data-testid="event-count"]');
  }

  get tailHash() {
    return $('[data-testid="tail-hash"]');
  }

  get refreshButton() {
    return $('button*=Aktualisieren');
  }

  get loadMoreButton() {
    return $('button*=Mehr laden');
  }

  get filterDropdown() {
    return $('[data-testid="event-filter"]');
  }

  get searchInput() {
    return $('[data-testid="audit-search"]');
  }

  // =============
  // Event Details
  // =============

  /**
   * Get audit event row by index
   */
  async getEventByIndex(index: number): Promise<WebdriverIO.Element> {
    const events = await this.auditEvents;
    return events[index];
  }

  /**
   * Get event timestamp
   */
  async getEventTimestamp(eventElement: WebdriverIO.Element): Promise<string> {
    const ts = await eventElement.$('[data-testid="event-timestamp"]');
    return await ts.getText();
  }

  /**
   * Get event type
   */
  async getEventType(eventElement: WebdriverIO.Element): Promise<string> {
    const type = await eventElement.$('[data-testid="event-type"]');
    return await type.getText();
  }

  /**
   * Get event self hash
   */
  async getEventSelfHash(eventElement: WebdriverIO.Element): Promise<string> {
    const hash = await eventElement.$('[data-testid="event-self-hash"]');
    return await hash.getText();
  }

  /**
   * Get event prev hash
   */
  async getEventPrevHash(eventElement: WebdriverIO.Element): Promise<string> {
    const hash = await eventElement.$('[data-testid="event-prev-hash"]');
    return await hash.getText();
  }

  /**
   * Expand event details
   */
  async expandEvent(eventElement: WebdriverIO.Element): Promise<void> {
    const expandBtn = await eventElement.$('[data-testid="expand-event"]');
    await expandBtn.click();
    await browser.pause(300);
  }

  // =============
  // Actions
  // =============

  /**
   * Navigate to audit view
   */
  async navigateToAudit(): Promise<void> {
    const auditLink = await $('a*=Audit');
    if (await auditLink.isExisting()) {
      await auditLink.click();
    }
    await this.auditView.waitForDisplayed({ timeout: 10000 });
  }

  /**
   * Check if audit timeline is visible
   */
  async isTimelineVisible(): Promise<boolean> {
    const timeline = await this.auditTimeline;
    return await timeline.isDisplayed();
  }

  /**
   * Get total number of events displayed
   */
  async getDisplayedEventCount(): Promise<number> {
    const events = await this.auditEvents;
    return events.length;
  }

  /**
   * Get total event count from header
   */
  async getTotalEventCount(): Promise<number> {
    const count = await this.eventCount;
    const text = await count.getText();
    const match = text.match(/(\d+)/);
    return match ? parseInt(match[1], 10) : 0;
  }

  /**
   * Verify hash chain integrity
   */
  async verifyHashChain(): Promise<void> {
    const verifyBtn = await this.verifyChainButton;
    await verifyBtn.waitForClickable({ timeout: 5000 });
    await verifyBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Check if hash chain is valid
   */
  async isChainValid(): Promise<boolean> {
    const status = await this.chainStatus;
    const text = await status.getText();
    return (
      text.toLowerCase().includes('gültig') ||
      text.toLowerCase().includes('valid') ||
      text.toLowerCase().includes('intakt')
    );
  }

  /**
   * Get chain verification status text
   */
  async getChainStatusText(): Promise<string> {
    const status = await this.chainStatus;
    return await status.getText();
  }

  /**
   * Get tail hash of the chain
   */
  async getTailHash(): Promise<string> {
    const hash = await this.tailHash;
    return await hash.getText();
  }

  /**
   * Refresh audit log
   */
  async refresh(): Promise<void> {
    const refreshBtn = await this.refreshButton;
    await refreshBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Load more events (pagination)
   */
  async loadMore(): Promise<void> {
    const loadMoreBtn = await this.loadMoreButton;
    if (await loadMoreBtn.isExisting()) {
      await loadMoreBtn.click();
      await this.waitForLoadingComplete();
    }
  }

  /**
   * Filter events by type
   */
  async filterByType(eventType: string): Promise<void> {
    const filter = await this.filterDropdown;
    await filter.selectByVisibleText(eventType);
    await browser.pause(500);
  }

  /**
   * Search events
   */
  async searchEvents(query: string): Promise<void> {
    const search = await this.searchInput;
    await search.setValue(query);
    await browser.pause(500);
  }

  /**
   * Get all events as structured data
   */
  async getAllEvents(): Promise<
    Array<{
      timestamp: string;
      type: string;
      selfHash: string;
      prevHash: string;
    }>
  > {
    const events = await this.auditEvents;
    const result: Array<{
      timestamp: string;
      type: string;
      selfHash: string;
      prevHash: string;
    }> = [];

    for (const event of events) {
      result.push({
        timestamp: await this.getEventTimestamp(event),
        type: await this.getEventType(event),
        selfHash: await this.getEventSelfHash(event),
        prevHash: await this.getEventPrevHash(event),
      });
    }

    return result;
  }

  /**
   * Verify that each event's prevHash matches the previous event's selfHash
   */
  async verifyChainLocally(): Promise<boolean> {
    const events = await this.getAllEvents();

    // Events should be in reverse chronological order (newest first)
    for (let i = 0; i < events.length - 1; i++) {
      const currentEvent = events[i];
      const previousEvent = events[i + 1];

      // Current event's prevHash should match previous event's selfHash
      if (currentEvent.prevHash !== previousEvent.selfHash) {
        console.error(
          `Chain break at index ${i}: prevHash ${currentEvent.prevHash} !== selfHash ${previousEvent.selfHash}`
        );
        return false;
      }
    }

    return true;
  }

  /**
   * Complete audit verification workflow
   */
  async completeAuditVerification(): Promise<{
    valid: boolean;
    eventCount: number;
    tailHash: string;
  }> {
    await this.navigateToAudit();
    await this.verifyHashChain();

    return {
      valid: await this.isChainValid(),
      eventCount: await this.getTotalEventCount(),
      tailHash: await this.getTailHash(),
    };
  }
}

export default new AuditPage();
