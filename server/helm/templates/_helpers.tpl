{{/* Expand the name of the chart. */}}
{{- define "airshipper.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/* Create a default fully qualified app name. truncated to 63 chars for k8s DNS, dont duplicate with chart name */}}
{{- define "airshipper.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}


{{/* airshipper Resource Names */}}

{{- define "airshipper.service" -}}
{{- (printf "%s" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.deployment" -}}
{{- (printf "%s" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.deploymentKind" -}}
{{- if .Values.persistence.enabled -}}StatefulSet{{- else -}}Deployment{{- end }}
{{- end }}

{{- define "airshipper.serviceAccount" -}}
{{- (printf "%s-serviceaccount" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.configMapSettings" -}}
{{- (printf "%s-settings" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.secret" -}}
{{- (printf "%s-secret" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.ingressSecret" -}}
{{- (printf "%s-ingresssecret" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.ingress" -}}
{{- (printf "%s-ingress" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.roleName" -}}
{{- (printf "%s-role" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.roleBindingName" -}}
{{- (printf "%s-rolebinding" (include "airshipper.fullname" .)) }}
{{- end }}

{{- define "airshipper.ingressSecretName" -}}
{{- (printf "%s-ingresssecret" (include "airshipper.fullname" .)) }}
{{- end }}

{{/* Common labels */}}
{{- define "airshipper.labels" -}}
{{ include "airshipper.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/* Selector labels */}}
{{- define "airshipper.selectorLabels" -}}
app.kubernetes.io/name: {{ include "airshipper.deployment" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}