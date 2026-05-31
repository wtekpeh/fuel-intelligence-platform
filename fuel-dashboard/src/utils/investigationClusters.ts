import type { InvestigationTimelineItem } from "./investigationTimeline";

export interface InvestigationCluster {
  id: string;
  startTime: string;
  endTime: string;
  items: InvestigationTimelineItem[];
}

const CLUSTER_WINDOW_MINUTES = 5;

export function buildInvestigationClusters(
  timelineItems: InvestigationTimelineItem[],
): InvestigationCluster[] {
  const clusters: InvestigationCluster[] = [];

  for (const item of timelineItems) {
    const itemTime = new Date(item.timestamp).getTime();

    const latestCluster = clusters[clusters.length - 1];

    if (!latestCluster) {
      clusters.push({
        id: `cluster-${item.timestamp}`,
        startTime: item.timestamp,
        endTime: item.timestamp,
        items: [item],
      });

      continue;
    }

    const latestClusterStartTime = new Date(latestCluster.startTime).getTime();

    const differenceMinutes = Math.abs(
      (latestClusterStartTime - itemTime) / 60000,
    );

    if (differenceMinutes <= CLUSTER_WINDOW_MINUTES) {
      latestCluster.items.push(item);
      latestCluster.endTime = item.timestamp;

      continue;
    }

    clusters.push({
      id: `cluster-${item.timestamp}`,
      startTime: item.timestamp,
      endTime: item.timestamp,
      items: [item],
    });
  }

  return clusters;
}
