function app() {
  return {
    tab: "channels",
    channels: [],
    logs: [],
    settings: { polling_interval: 300 },
    form: { channel_id: "", channel_name: "", webhook_url: "" },
    loading: false,
    polling: false,

    async init() {
      await this.fetchChannels();
      await this.fetchSettings();
    },

    async fetchChannels() {
      const res = await fetch("/api/channels");
      this.channels = await res.json();
    },

    async addChannel() {
      this.loading = true;
      await fetch("/api/channels", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(this.form),
      });
      this.form = { channel_id: "", channel_name: "", webhook_url: "" };
      await this.fetchChannels();
      this.loading = false;
    },

    async deleteChannel(id) {
      await fetch(`/api/channels/${id}`, { method: "DELETE" });
      await this.fetchChannels();
    },

    async fetchLogs() {
      const res = await fetch("/api/logs");
      this.logs = await res.json();
    },

    async fetchSettings() {
      const res = await fetch("/api/settings");
      this.settings = await res.json();
    },

    async saveSettings() {
      await fetch("/api/settings", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(this.settings),
      });
    },

    async triggerPoll() {
      this.polling = true;
      await fetch("/api/poll", { method: "POST" });
      this.polling = false;
    },
  };
}
