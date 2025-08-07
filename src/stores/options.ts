import { defineStore } from 'pinia';
import { DateTime } from 'luxon';

const format = 'yyyy/MM/dd';
const now = DateTime.now().minus({ days: 1 }); // Start from yesterday to match Steam's dashboards

const periods = [{
  label: "Today",
  value: "today"
}, {
  label: "This week",
  value: "1w"
}, {
  label: "Last two weeks",
  value: "2w"
}, {
  label: "This month",
  value: "1m"
}, {
  label: "Last three months",
  value: "3m"
}, {
  label: "This year",
  value: "1y"
}, {
  label: "All Time",
  value: ""

}]
type Options = {
  loading: boolean;
  period: "today" | "1w" | "2w" | "1m" | "3m" | "1y" | "";
  periods: typeof periods;
  salesType: "gross" | "net";
};

export const useOptionsStore = defineStore('options', {
  state: () => ({
    loading: false,
    period: "1m",
    periods,
    salesType: "gross"
  } as Options),
  getters: {
    isLoading(state): boolean {
      return state.loading;
    },
    minusDays(state): number {
      const period = state.period;
      switch (period) {
        case "today":
          return -1;
        case "1w":
          return 7;
        case "2w":
          return 14;
        case "1m":
          return 30;
        case "3m":
          return 90;
        case "1y":
          return 365;
        case "":
          return 0;
        default:
          return 0;
      }
    },
    now() {
      return now.toFormat(format);
    },
    fromDateTime() {
      if (this.minusDays == 0) return null
      return now.minus({ days: this.minusDays });
    },
    toDateTime() {
      if (this.minusDays == 0) return null;

      if (this.minusDays == -1) {
        return now.plus({ days: 1 });
      } else return now;
    },
    from() {
      return this.fromDateTime?.toFormat(format)
    },
    to() {
      return this.toDateTime?.toFormat(format)
    },
    previousFrom() {
      if (this.minusDays == 0) return null;
      const previous = now.minus({ days: (this.minusDays + 1) * 2 });
      return previous.toFormat(format)
    },
    previousTo() {
      if (this.minusDays == 0) return null;
      const previous = now.minus({ days: this.minusDays + 1 });
      return previous.toFormat(format)
    },
  },
  actions: {
    isInPeriod(date: string): boolean {
      return date >= this.from && date <= this.to;
    },
    isInPreviousPeriod(date: string): boolean {
      return date >= this.previousFrom && date <= this.previousTo;
    }
  },
})

export default useOptionsStore;