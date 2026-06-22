// routes.js - SPA routes for the dashboard.
//
// One dashboard view, addressable per organization so the org tabs are real links and
// the back button works. Hash mode keeps it working on any static host (and on the
// device's own server) with no catch-all rewrite rule.

export const routes = [
  { path: '/', component: 'dashboard-page' },
  { path: '/org/:id', component: 'dashboard-page' },
];
