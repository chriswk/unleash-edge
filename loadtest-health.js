import http from 'k6/http';

export default function () {
  http.get('http://192.168.1.42:3001/internal-backstage/health');
}
