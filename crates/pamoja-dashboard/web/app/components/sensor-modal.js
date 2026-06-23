// sensor-modal.js - the full-screen sensor detail (opened from the grid or an alarm).
//
// Mounted at the body level, above everything. Driven by store.selected. Opening from
// the network map instead uses a docked panel (see network-view); this modal is for the
// grid and the alarm list. Closes via the ✕, backdrop, Back, or Escape - all through
// nav so history stays balanced. The body is the shared sensor detail.

import { store } from '../store.js';
import { currentFleet } from '../lib/edits.js';
import { back } from '../nav.js';
import { sensorDetailBody, stickLog } from '../lib/detail.js';
import { t } from '../lib/i18n.js';
import { conn, esc } from '../lib/viz/index.js';

$.component('sensor-modal', {
  /** Re-renders on store changes and on each fleet frame. */
  mounted()
  {
    this._un = store.subscribe(() => this.setState({}));
    this._eff = $.effect(() => { currentFleet(); this.setState({}); });
  },
  /** Tears down the store subscription and fleet effect. */
  destroyed() { if (this._un) this._un(); if (typeof this._eff === 'function') this._eff(); },
  /** Keeps the event log pinned to its newest line after a re-render. */
  updated() { stickLog(this._el); },

  /** Closes the modal by unwinding one history entry. */
  close() { back(); },
  /**
   * Closes the modal when the backdrop itself is clicked.
   *
   * @param {MouseEvent} e - the click event.
   * @returns {void}
   */
  onOverlay(e) { if (e.target.classList.contains('modal-overlay')) back(); },

  /**
   * Resolves the selected sensor and its org/group from the current fleet.
   *
   * @returns {{org: object, group: object, sensor: object}|null} the selection, or null.
   */
  find()
  {
    const sel = store.state.selected; const f = currentFleet();
    if (!sel || !f) return null;
    const [gid, sid] = sel.split('/');
    for (const o of f.orgs) for (const g of o.groups) { if (g.id !== gid) continue; const s = g.sensors.find((x) => x.id === sid); if (s) return { org: o, group: g, sensor: s }; }
    return null;
  },

  /**
   * Renders the modal for the selected sensor, or an empty placeholder when none.
   *
   * @returns {string} the modal markup.
   */
  render()
  {
    const found = this.find();
    if (!found) return '<div hidden></div>';
    const { org, group, sensor: s } = found;
    return `
      <div class="modal-overlay" @click="onOverlay">
        <div class="modal" data-status="${s.reading.status}" role="dialog" aria-modal="true">
          <div class="modal-head">
            <div class="modal-head-main">
              <div class="modal-title">${esc(t('label.' + s.reading.key))}</div>
              <div class="modal-sub">${esc(org.name)} · ${esc(group.name)}</div>
            </div>
            <div class="modal-head-side">${conn(group.link)}<button class="modal-close" type="button" @click="close" aria-label="Close">✕</button></div>
          </div>
          ${sensorDetailBody(s)}
        </div>
      </div>`;
  },
});
