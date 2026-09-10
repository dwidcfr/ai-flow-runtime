import dagre from 'dagre'
import {
  Background,
  Controls,
  Handle,
  MiniMap,
  Position,
  ReactFlow,
  addEdge,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
  type NodeProps,
  type NodeTypes,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import type { DraftFlowDto, FlowNodeDto } from '@/shared/types'

const NODE_WIDTH = 180
const NODE_HEIGHT = 60

export type FlowNodeData = {
  label: string
  nodeType: string
}

export type FlowNode = Node<FlowNodeData>

function FlowNodeCard({ data }: NodeProps<FlowNode>) {
  return (
    <div style={{
      padding: '8px 12px',
      borderRadius: 8,
      background: '#2a2a2a',
      border: '1px solid #444',
      minWidth: NODE_WIDTH,
      fontSize: 12,
    }}>
      <Handle type="target" position={Position.Top} />
      <div style={{ fontWeight: 600 }}>{data.label}</div>
      <div style={{ color: '#aaa' }}>{data.nodeType}</div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  )
}

const nodeTypes: NodeTypes = { flowNode: FlowNodeCard }

export function flowToReactFlow(flow: DraftFlowDto): { nodes: FlowNode[]; edges: Edge[] } {
  const nodes: FlowNode[] = flow.nodes.map((n) => ({
    id: n.id,
    type: 'flowNode',
    position: n.position ? { x: n.position.x, y: n.position.y } : { x: 0, y: 0 },
    data: { label: n.id, nodeType: n.node_type },
  }))

  const edges: Edge[] = []
  for (const node of flow.nodes) {
    for (const t of node.transitions) {
      edges.push({
        id: `${node.id}-${t.action}-${t.target}`,
        source: node.id,
        target: t.target,
        label: t.action,
        animated: true,
      })
    }
  }
  return { nodes, edges }
}

export function reactFlowToDraft(
  flow: DraftFlowDto,
  nodes: FlowNode[],
  edges: Edge[],
): DraftFlowDto {
  const edgeMap = new Map<string, { action: string; target: string }[]>()
  for (const edge of edges) {
    const list = edgeMap.get(edge.source) ?? []
    list.push({ action: String(edge.label ?? 'next'), target: edge.target })
    edgeMap.set(edge.source, list)
  }

  const flowNodes: FlowNodeDto[] = nodes.map((n) => {
    const existing = flow.nodes.find((fn) => fn.id === n.id)
    return {
      id: n.id,
      node_type: existing?.node_type ?? String(n.data.nodeType ?? 'message'),
      payload: existing?.payload ?? {},
      transitions: edgeMap.get(n.id) ?? existing?.transitions ?? [],
      position: { x: n.position.x, y: n.position.y },
    }
  })

  return {
    ...flow,
    nodes: flowNodes,
    initial: flow.initial || flowNodes[0]?.id || '',
  }
}

export function autoLayout(nodes: FlowNode[], edges: Edge[]): FlowNode[] {
  const g = new dagre.graphlib.Graph()
  g.setDefaultEdgeLabel(() => ({}))
  g.setGraph({ rankdir: 'TB', nodesep: 50, ranksep: 80 })

  nodes.forEach((n) => g.setNode(n.id, { width: NODE_WIDTH, height: NODE_HEIGHT }))
  edges.forEach((e) => g.setEdge(e.source, e.target))

  dagre.layout(g)

  return nodes.map((n) => {
    const pos = g.node(n.id)
    return {
      ...n,
      position: { x: pos.x - NODE_WIDTH / 2, y: pos.y - NODE_HEIGHT / 2 },
    }
  })
}

export { useNodesState, useEdgesState, addEdge, ReactFlow, Background, Controls, MiniMap, nodeTypes }
export type { Connection, Edge }
