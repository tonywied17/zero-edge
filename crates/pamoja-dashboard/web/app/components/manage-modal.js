// manage-modal.js - create a group or add a sensor (demo provisioning).
//
// Driven by store.create ({ mode:'group', orgId } or { mode:'sensor', groupId }). The
// type and link-kind choices come from the layout catalog (see lib/catalog.js). The
// edits are client-side only (see lib/edits.js); a real build would post these to the
// device as authenticated provisioning commands.

import { store } from '../store.js';
import { back } from '../nav.js';
import { t } from '../lib/i18n.js';
import { makeGroup, makeSensor, currentFleet } from '../lib/edits.js';
import { catalog } from '../lib/catalog.js';
import { LINK_NAMES, esc } from '../lib/viz/index.js';

$.component('manage-modal', {
  state: { name: '', linkKind: 'lora', sensorKind: 'temperature', value: '', last: null },

  /** Resets the form whenever the create target changes. */
  mounted() { this._un = store.subscribe(() => this.sync()); },
  /** Tears down the store subscription. */
  destroyed() { if (this._un) this._un(); },

  /** Resets the form fields when a new create dialog opens, then re-renders. */
  sync()
  {
    const c = store.state.create;
    const id = c ? c.mode + (c.orgId || c.groupId) : null;
    if (id !== this.state.last)
    {
      this.state.last = id;
      this.state.name = '';
      this.state.value = '';
      this.state.linkKind = 'lora';
      this.state.sensorKind = 'temperature';
    }
    this.setState({});
  },

  /**
   * Selects a link kind for the new group.
   *
   * @param {string} k - the chosen link kind.
   * @returns {void}
   */
  setLink(k) { this.state.linkKind = k; },
  /**
   * Selects a sensor preset for the new sensor.
   *
   * @param {string} k - the chosen preset id.
   * @returns {void}
   */
  setKind(k) { this.state.sensorKind = k; },

  /**
   * The sensor presets offered for a target group; mesh-only presets appear only when the
   * target group is on a mesh link.
   *
   * @param {string} groupId - the target group's id.
   * @returns {Array<object>} the offered presets.
   */
  presetsFor(groupId)
  {
    const f = currentFleet();
    let kind = null;
    if (f) for (const o of f.orgs) for (const g of o.groups) if (g.id === groupId) kind = g.link.kind;
    return catalog.sensorPresets.filter((p) => !p.meshOnly || kind === 'mesh');
  },

  /** Cancels the dialog by unwinding one history entry. */
  cancel() { back(); },
  /**
   * Cancels the dialog when the backdrop itself is clicked.
   *
   * @param {MouseEvent} e - the click event.
   * @returns {void}
   */
  onOverlay(e) { if (e.target.classList.contains('modal-overlay')) back(); },

  /** Creates the group or sensor from the form, then closes the dialog. */
  submit()
  {
    const c = store.state.create; if (!c) return;
    if (c.mode === 'group')
    {
      store.dispatch('addGroup', makeGroup(c.orgId, this.state.name.trim() || t('ui.newGroup'), this.state.linkKind));
    } else
    {
      const v = parseFloat(this.state.value);
      store.dispatch('addSensor', makeSensor(c.groupId, this.state.sensorKind, Number.isFinite(v) ? v : NaN));
    }
    back();
  },

  /**
   * Renders the create dialog for the active target, or an empty placeholder when none.
   *
   * @returns {string} the dialog markup.
   */
  render()
  {
    const c = store.state.create;
    if (!c) return '<div hidden></div>';
    const s = this.state;
    const body = c.mode === 'group'
      ? `
        <label class="field"><span>${t('ui.name')}</span>
          <input class="field-input" type="text" z-model="name" placeholder="${esc(t('ui.newGroup'))}" /></label>
        <div class="field"><span>${t('ui.connection')}</span>
          <div class="chips">${catalog.linkKinds.map((k) => `<button type="button" class="chip-opt ${s.linkKind === k ? 'on' : ''}" @click="setLink('${k}')">${esc(LINK_NAMES[k] || k)}</button>`).join('')}</div>
        </div>`
      : `
        <div class="field"><span>${t('ui.type')}</span>
          <div class="chips">${this.presetsFor(c.groupId).map((p) => `<button type="button" class="chip-opt ${s.sensorKind === p.id ? 'on' : ''}" @click="setKind('${p.id}')">${esc(t('label.' + p.key))}</button>`).join('')}</div>
        </div>
        <label class="field"><span>${t('ui.value')}</span>
          <input class="field-input" type="number" step="any" z-model="value" placeholder="${t('ui.auto')}" /></label>`;
    return `
      <div class="modal-overlay" @click="onOverlay">
        <div class="modal modal-form" role="dialog" aria-modal="true">
          <div class="modal-head">
            <div class="modal-title">${c.mode === 'group' ? t('ui.addGroup') : t('ui.addSensor')}</div>
            <button class="modal-close" type="button" @click="cancel" aria-label="${esc(t('ui.cancel'))}">✕</button>
          </div>
          <div class="form">${body}</div>
          <div class="form-actions">
            <button class="seg" type="button" @click="cancel">${t('ui.cancel')}</button>
            <button class="seg primary" type="button" @click="submit">${t('ui.create')}</button>
          </div>
        </div>
      </div>`;
  },
});
